use sqlx::{ 
    QueryBuilder,
    SqlitePool,
    Sqlite,
    FromRow,
    Row,
};

use crate::domain::{
    rule_model::{WeeklyRule, RuleAssignment},
    shift_calendar_model::{
        WeekStatus,
        PlanId,
        ShiftCalendarManager
    }
};

pub struct CalendarRepository {
    pool: SqlitePool,
}

// =====================
// DB読み込み用ヘルパー構造体
// =====================

// 親テーブル読み込み用
#[derive(FromRow)]
struct CalendarHeaderRow {
    id: i64,
    plan_id: PlanId,
    base_abs_week: i64,
    initial_delta: i64,
}

// 子テーブル読み込み用
#[derive(FromRow)]
struct WeekStatusRow {
    #[allow(dead_code)] // クエリで取得するがRust側で使わない場合用
    week_offset: i64,
    rule_id: Option<i64>,
    status_type: String,
    logical_delta: Option<i64>,
}

use serde::Serialize;

// フロントエンドやロジック層に渡すための「リッチな」構造体
#[derive(Debug, Serialize)]
pub struct WeeklyRuleWithAssignments {
    // #[serde(flatten)] をつけると、JSON化したときに rule の中身がトップレベルに展開されます
    // { "id": 1, "name": "RuleA", "assignments": [...] } となり使いやすいです
    #[serde(flatten)] 
    pub rule: WeeklyRule,
    pub assignments: Vec<RuleAssignment>,
}

// ★ ここに変換ロジックを書く
// "Row" は "Domain Model" になれる (TryFrom)
impl TryFrom<WeekStatusRow> for WeekStatus {
    type Error = String; // エラー型

    fn try_from(row: WeekStatusRow) -> Result<Self, Self::Error> {
        match row.status_type.as_str() {
            "Active" => Ok(WeekStatus::Active {
                logical_delta: row.logical_delta.ok_or("Active status missing delta")? as usize,
                rule_id: row.rule_id.ok_or("Active status missing rule_id")?,
            }),
            "Skipped" => Ok(WeekStatus::Skipped),
            other => Err(format!("Unknown status type: {}", other)),
        }
    }
}

impl CalendarRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 新しいシフトカレンダーを作成する
    /// すでに同じ plan_id のカレンダーが存在する場合はエラーを返す
    pub async fn create_calendar(
        &self,
        plan_id: i64,
        base_abs_week: usize,
        initial_delta: usize,
    ) -> Result<i64, String> {

        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        // 1. 既存カレンダーのチェック（重複作成の防止）
        let existing = sqlx::query("SELECT id FROM shift_calendars WHERE plan_id = ?")
            .bind(plan_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(row) = existing {
            let existing_id: i64 = row.get("id");
            // 既に存在する場合は、トランザクションをキャンセルしてエラーを返す
            return Err(format!(
                "Plan ID: {} のカレンダーは既に存在します (Calendar ID: {})", 
                plan_id, existing_id
            ));
        }

        // 2. 新規カレンダーの挿入 (INSERT)
        let new_calendar_id = sqlx::query(
            "INSERT INTO shift_calendars (plan_id, base_abs_week, initial_delta) 
             VALUES (?, ?, ?)"
        )
        .bind(plan_id)
        .bind(base_abs_week as i64)
        .bind(initial_delta as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();

        tx.commit().await.map_err(|e| e.to_string())?;

        // 成功した場合は、新しく作られたIDを返す
        Ok(new_calendar_id)
    }

    pub async fn try_to_append_timeline(
        &self,
        plan_id: i64,
        start_abs_week: usize,
        status_iterator: impl IntoIterator<Item = Option<i64>>,
    ) -> Result<(), String> {

        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        // 1. カレンダー情報の取得
        let cal_row = sqlx::query("SELECT id, base_abs_week FROM shift_calendars WHERE plan_id = ?")
            .bind(plan_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        let cal = cal_row.ok_or_else(|| format!("Plan ID: {} のカレンダーが存在しません。", plan_id))?;
        let calendar_id: i64 = cal.get("id");
        let base_abs_week: i64 = cal.get("base_abs_week");

        // 2. 現在のDBの「末尾（cursor）」と「logical_delta」を取得
        // DBが空（作成直後）の場合は -1 となる
        let row = sqlx::query(
            "SELECT MAX(week_offset) as max_offset, MAX(logical_delta) as max_logical_delta
             FROM weekly_statuses WHERE calendar_id = ?"
        )
        .bind(calendar_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let current_db_cursor: i64 = row.try_get("max_offset").unwrap_or(None).unwrap_or(-1);
        let mut local_logical_delta: i64 = row.try_get("max_logical_delta").unwrap_or(None).unwrap_or(-1);

        // 今回のリストの開始オフセット
        let start_offset = (start_abs_week as i64) - base_abs_week;

        // ====================================================================
        // ★ バリデーションとスキップ計算
        // ====================================================================

        // 【仕様】 歯抜けエラーのチェック
        // カレンダー作成直後 (cursor=-1) の場合、start_offset は必ず 0 (base_abs_week) でなければエラーになる
        if start_offset > current_db_cursor + 1 {
            let missing_week = base_abs_week as i64 + current_db_cursor + 1;
            return Err(format!(
                "タイムラインに空きがあります。絶対週 {} からデータを連続させてください。",
                missing_week
            ));
        }

        // かぶっている要素数を計算
        let overlap_count = std::cmp::max(0, current_db_cursor - start_offset + 1);

        // かぶっている分をスキップした新しいイテレータ
        let new_items = status_iterator.into_iter().skip(overlap_count as usize);

        // 実際の INSERT 開始位置
        let insert_start_offset = start_offset + overlap_count;

        // 3. ループ処理：残った未来の要素だけをINSERT
        for (index, status_opt) in new_items.enumerate() {
            let target_offset = insert_start_offset + index as i64;

            let (st_type, delta_to_save, r_id) = match status_opt {
                Some(rule_id) => {
                    local_logical_delta += 1;
                    ("Active", Some(local_logical_delta), Some(rule_id))
                },
                None => ("Skipped", None, None),
            };

            sqlx::query(
                "INSERT INTO weekly_statuses (calendar_id, week_offset, status_type, logical_delta, rule_id)
                 VALUES (?, ?, ?, ?, ?)"
            )
            .bind(calendar_id)
            .bind(target_offset)
            .bind(st_type)
            .bind(delta_to_save)
            .bind(r_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn find_by_plan_id(&self, plan_id: i64) -> Result<Option<ShiftCalendarManager>, String> {
        let header_opt: Option<CalendarHeaderRow> = sqlx::query_as::<Sqlite, CalendarHeaderRow>("
            SELECT id, plan_id, base_abs_week, initial_delta 
            FROM shift_calendars 
            WHERE plan_id = ? LIMIT 1")
            .bind(plan_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let header = match header_opt {
            Some(h) => h,
            None => return Ok(None),
        };

        let rows: Vec<WeekStatusRow> = sqlx::query_as("
            SELECT week_offset, status_type, logical_delta, rule_id 
            FROM weekly_statuses 
            WHERE calendar_id = ?
            ORDER BY week_offset ASC")
            .bind(header.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let timeline = rows.into_iter().map(|row| row.try_into()).collect::<Result<Vec<_>,_>>()?;

        Ok(Some(ShiftCalendarManager {
            id: Some(header.id),
            plan_id: header.plan_id,
            base_abs_week: header.base_abs_week as usize,
            initial_delta: header.initial_delta as usize,
            timeline,
        }))
    }

    /// 指定された範囲（offset start から count 分）のステータスだけを取得
    pub async fn fetch_status_range(
        &self,
        calendar_id: i64,
        start_offset: i64,
        count: i64
    ) -> Result<Vec<WeekStatus>, String> {
        let rows = sqlx::query_as::<_, WeekStatusRow>(
            "SELECT week_offset, status_type, logical_delta, rule_id
             FROM weekly_statuses
             WHERE calendar_id = ? AND week_offset >= ? AND week_offset < ?
             ORDER BY week_offset ASC"
        )
        .bind(calendar_id)
        .bind(start_offset)
        .bind(start_offset + count)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // DTO -> Domain Model 変換 (省略)
        let statuses = rows.into_iter()
            .map(|row| row.try_into()) // ★ ここで変換が走る
            .collect::<Result<Vec<WeekStatus>, String>>()?;

        Ok(statuses)
    }

    /// IDリストに含まれるルールだけを取得 (IN句を使用)
    pub async fn fetch_rules_by_ids(
        &self,
        rule_ids: &[i64]
    ) -> Result<Vec<WeeklyRuleWithAssignments>, String> {
        if rule_ids.is_empty() {
            return Ok(vec![]);
        }

        // ---------------------------------------------------
        // 1. ルール本体の一括取得 (QueryBuilderを使用)
        // ---------------------------------------------------
        // "SELECT * FROM weekly_rules WHERE id IN (" までを作成
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, plan_id, name, sort_order FROM weekly_rules WHERE id IN ("
        );

        // rule_ids をカンマ区切りでバインドしていく
        // separated(", ") が自動的にカンマを入れてくれます
        let mut separated = query_builder.separated(", ");
        for id in rule_ids {
            separated.push_bind(id);
        }

        // 閉じ括弧
        separated.push_unseparated(")");

        // 実行
        let rules = query_builder
            .build_query_as::<WeeklyRule>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        // ---------------------------------------------------
        // 2. Assignmentsの一括取得 (N+1問題の解消)
        // ---------------------------------------------------
        // 同じ rule_ids を使って、関連する Assignment も一度に取ってくる

        let mut assign_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT 
                id,
                weekly_rule_id,
                weekday,
                shift_time_type,
                target_group_id,
                target_member_index
             FROM rule_assignments
             WHERE weekly_rule_id IN ("
        );

        let mut assign_sep = assign_builder.separated(", ");
        for id in rule_ids {
            assign_sep.push_bind(id);
        }
        assign_sep.push_unseparated(")");

        let assignments = assign_builder
            .build_query_as::<RuleAssignment>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        // ---------------------------------------------------
        // 3. メモリ上で結合 (Group By)
        // ---------------------------------------------------
        // 取得した Assignments を rule_id ごとに振り分ける
        // HashMap<rule_id, Vec<Assignment>> を作ってもいいですが、
        // 単純なフィルタリングでも可読性は高いです（件数が少なければ）

        let mut result = Vec::new();

        for rule in rules {
            // このルールに属する assignment だけを抽出
            // (効率化するならHashMap化推奨ですが、カレンダー1画面分ならこれで十分高速です)
            let related_assignments: Vec<RuleAssignment> = assignments
                .iter()
                .filter(|a| a.weekly_rule_id == rule.id)
                .cloned()
                .collect();

            result.push(WeeklyRuleWithAssignments {
                rule,
                assignments: related_assignments,
            });
        }

        Ok(result)
    }

    /// デバッグ用：指定したプランのタイムラインデータをDBから取得して表示する
    pub async fn debug_print_timeline(&self, plan_id: i64) -> Result<(), String> {
        let mut conn = self.pool.acquire().await.map_err(|e| e.to_string())?;

        // 1. まずカレンダーの基本情報を取得
        let cal_row = sqlx::query(
            "SELECT id, base_abs_week FROM shift_calendars WHERE plan_id = ?"
        )
        .bind(plan_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;

        // カレンダーが存在しない場合は終了
        let cal_row = match cal_row {
            Some(row) => row,
            None => {
                println!("--- [DEBUG] Plan ID: {} のカレンダーは見つかりませんでした ---", plan_id);
                return Ok(());
            }
        };

        let calendar_id: i64 = cal_row.get("id");
        let base_abs_week: i64 = cal_row.get("base_abs_week");

        println!("--- [DEBUG] Timeline for Plan ID: {} (Calendar ID: {}) ---", plan_id, calendar_id);
        println!("基準絶対週 (base_abs_week): {}", base_abs_week);
        println!("---------------------------------------------------------------");
        println!("| Offset | Absolute Week | Status  | Logical Delta | Rule ID  |");
        println!("---------------------------------------------------------------");

        // 2. タイムライン(weekly_statuses)を offset 順に取得
        let status_rows = sqlx::query(
            "SELECT week_offset, status_type, logical_delta, rule_id
             FROM weekly_statuses
             WHERE calendar_id = ?
             ORDER BY week_offset ASC"
        )
        .bind(calendar_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;

        if status_rows.is_empty() {
            println!("| (データなし)                                                |");
        } else {
            // 3. 取得したデータを1行ずつ表示
            for row in status_rows {
                let offset: i64 = row.get("week_offset");
                let status_type: String = row.get("status_type");

                // NULLになり得るカラムは Option で取得
                let logical_delta: Option<i64> = row.try_get("logical_delta").unwrap_or(None);
                let rule_id: Option<i64> = row.try_get("rule_id").unwrap_or(None);

                // 絶対週の計算 (base + offset)
                let abs_week = base_abs_week + offset;

                // 見やすくフォーマット
                let delta_str = logical_delta.map_or("-".to_string(), |v| v.to_string());
                let rule_str = rule_id.map_or("-".to_string(), |v| v.to_string());

                println!(
                    "| {:<6} | {:<13} | {:<7} | {:<13} | {:<8} |",
                    offset, abs_week, status_type, delta_str, rule_str
                );
            }
        }
        println!("---------------------------------------------------------------");

        Ok(())
    }
}

