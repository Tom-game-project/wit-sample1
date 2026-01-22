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

    /// タイムラインを保存（追記）する
    /// 1件ずつ順番にチェックし、未保存の続きデータであればINSERTする
    pub async fn save_timeline(
        &self,
        plan_id: i64,
        base_abs_week: usize,
        initial_delta: usize,
        timeline: &[WeekStatus], // Managerではなく、データ配列そのものを受け取る
    ) -> Result<(), String> {

        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        // 1. 親カレンダーIDの取得 (なければ作成)
        //    ここもシンプルに「SELECTしてなければINSERT」の流れにします
        let calendar_id_opt: Option<i64> = sqlx::query("SELECT id FROM shift_calendars WHERE plan_id = ?")
            .bind(plan_id)
            .fetch_optional(&mut *tx).await.map_err(|e| e.to_string())?
            .map(|row| row.get("id"));

        let calendar_id = if let Some(id) = calendar_id_opt {
            id
        } else {
            let new_id = sqlx::query("
                INSERT INTO shift_calendars (plan_id, base_abs_week, initial_delta) 
                VALUES (?, ?, ?)")
                .bind(plan_id)
                .bind(base_abs_week as i64)
                .bind(initial_delta as i64)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?
                .last_insert_rowid();
            new_id
        };

        // 2. 現在のDB上の「到達点（最大オフセット）」を取得
        let row = sqlx::query("SELECT MAX(week_offset) as max_offset FROM weekly_statuses WHERE calendar_id = ?")
            .bind(calendar_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // ★修正: 明示的に Option<i64> として取得することで、NULLを確実にハンドリングする
        // try_get("max_offset") で i64 を指定すると、環境によっては NULL が 0 になる恐れがあるため
        let max_offset_opt: Option<i64> = row.try_get("max_offset").unwrap_or(None);

        // NULL(None)なら -1、値があればその値を使う
        let mut current_db_cursor: i64 = max_offset_opt.unwrap_or(-1);

        // 3. ループ処理: 1件ずつチェックしてINSERT
        for (index, status) in timeline.iter().enumerate() {
            let target_offset = index as i64;

            // 判定: DBのカーソルより先にあるデータだけを入れる
            // 例: DBに0,1,2がある(cursor=2)。targetが3なら入れる。targetが2なら無視。
            if target_offset > current_db_cursor {
                // INSERT実行
                let (st_type, delta, r_id) = match status {
                    WeekStatus::Active { logical_delta, rule_id } =>
                        ("Active", Some(*logical_delta as i64), Some(*rule_id)),
                    WeekStatus::Skipped =>
                        ("Skipped", None, None),
                };

                sqlx::query(
                    "INSERT INTO weekly_statuses (
                        calendar_id, 
                        week_offset, 
                        status_type, 
                        logical_delta,
                        rule_id)
                    VALUES (?, ?, ?, ?, ?)"
                )
                .bind(calendar_id /* calendar_id */)
                .bind(target_offset/* week_offset */)
                .bind(st_type/* status_type */)
                .bind(delta /* logical_delta */)
                .bind(r_id /* rule_id */)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;

                // カーソルを進める（これにより、次のループでの連続性が保証される）
                current_db_cursor = target_offset;
            } else {
                // すでにDBにあるので無視 (上書き防止)
                // continue;
            }
        }

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn save_calendar(&self, manager: &ShiftCalendarManager) -> Result<(), String> {
        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        // 既存削除
        sqlx::query("DELETE FROM shift_calendars WHERE plan_id = ?")
            .bind(manager.plan_id)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        // 親作成
        let row_id = sqlx::query("
            INSERT INTO shift_calendars (
                plan_id,
                base_abs_week,
                initial_delta)
            VALUES (?, ?, ?)")
            .bind(manager.plan_id)
            .bind(manager.base_abs_week as i64)
            .bind(manager.initial_delta as i64)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?
            .last_insert_rowid();

        // 子作成
        for (index, status) in manager.timeline.iter().enumerate() {
            let (st_type, delta, r_id) = match status {
                WeekStatus::Active { logical_delta, rule_id } => ("Active", Some(*logical_delta as i64), Some(*rule_id)),
                WeekStatus::Skipped => ("Skipped", None, None),
            };

            sqlx::query("
                INSERT INTO weekly_statuses (
                    calendar_id,
                    week_offset,
                    status_type,
                    logical_delta,
                    rule_id
                )
                VALUES (?, ?, ?, ?, ?)")
                .bind(row_id)
                .bind(index as i64)
                .bind(st_type)
                .bind(delta)
                .bind(r_id)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;
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
}

