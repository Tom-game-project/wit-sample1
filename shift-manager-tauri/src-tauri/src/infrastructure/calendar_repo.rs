use sqlx::{ 
    QueryBuilder,
    SqlitePool,
    Sqlite,
    FromRow,
};

use crate::domain::{
    rule_model::{WeeklyRule, RuleAssignment},
    shift_calendar_model::*
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
                rule_id: row.rule_id.ok_or("Active status missing rule_id")? as usize,
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
            "SELECT id, weekly_rule_id, weekday, shift_time_type, target_group_id, target_member_index
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

// calendar の操作は簡単にユーザーが月指定でできるように実装する
//

#[cfg(test)]
mod calendar_repo_tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use crate::domain::shift_calendar_model::WeekStatus;

    // 1. テスト用セットアップ
    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create memory pool");

        // ★ staff_groups テーブルを追加
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to create schema");

        pool
    }

    // 2. ヘルパー関数

    async fn create_plan(pool: &SqlitePool, name: &str) -> i64 {
        sqlx::query("INSERT INTO plans (name) VALUES (?)")
            .bind(name)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    // ★ 追加: グループ作成ヘルパー
    async fn create_group(pool: &SqlitePool, plan_id: i64, name: &str) -> i64 {
        sqlx::query("INSERT INTO staff_groups (plan_id, name, sort_order) VALUES (?, ?, 0)")
            .bind(plan_id)
            .bind(name)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn create_rule(pool: &SqlitePool, plan_id: i64, name: &str) -> i64 {
        sqlx::query("INSERT INTO weekly_rules (plan_id, name, sort_order) VALUES (?, ?, 0)")
            .bind(plan_id)
            .bind(name)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    // ★ 修正: group_id を引数で受け取る
    async fn create_assignment(pool: &SqlitePool, rule_id: i64, weekday: i64, group_id: i64) {
        sqlx::query(
            "INSERT INTO rule_assignments 
            (weekly_rule_id, weekday, shift_time_type, target_group_id, target_member_index) 
            VALUES (?, ?, 0, ?, 0)"
        )
        .bind(rule_id)
        .bind(weekday)
        .bind(group_id) // ★ ここに渡す
        .execute(pool)
        .await
        .unwrap();
    }

    // ... (create_calendar, create_status は変更なし) ...
    async fn create_calendar(pool: &SqlitePool, plan_id: i64) -> i64 {
        sqlx::query("INSERT INTO shift_calendars (plan_id, base_abs_week, initial_delta) VALUES (?, 100, 0)")
            .bind(plan_id)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn create_status(pool: &SqlitePool, cal_id: i64, offset: i64, status: &str, delta: Option<i64>, rule_id: Option<i64>) {
        sqlx::query(
            "INSERT INTO weekly_statuses (calendar_id, week_offset, status_type, logical_delta, rule_id) 
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(cal_id)
        .bind(offset)
        .bind(status)
        .bind(delta)
        .bind(rule_id)
        .execute(pool)
        .await
        .unwrap();
    }

    // 3. テストケース修正

    #[tokio::test]
    async fn test_fetch_rules_by_ids() {
        let pool = setup_test_db().await;
        let repo = CalendarRepository::new(pool.clone());

        let plan_id = create_plan(&pool, "Test Plan").await;
        
        // ★ ここでダミーグループを作成
        let group_id = create_group(&pool, plan_id, "Group A").await;
        
        // Rule A
        let rule_a_id = create_rule(&pool, plan_id, "Rule A").await;
        create_assignment(&pool, rule_a_id, 0, group_id).await; // Mon
        create_assignment(&pool, rule_a_id, 1, group_id).await; // Tue

        // Rule B
        let rule_b_id = create_rule(&pool, plan_id, "Rule B").await;
        create_assignment(&pool, rule_b_id, 5, group_id).await; // Sat

        // Rule C
        let rule_c_id = create_rule(&pool, plan_id, "Rule C").await;
        create_assignment(&pool, rule_c_id, 6, group_id).await; 

        // [Act]
        let results = repo.fetch_rules_by_ids(&[rule_a_id, rule_b_id]).await.expect("Failed to fetch");

        // [Assert]
        assert_eq!(results.len(), 2);

        let res_a = results.iter().find(|r| r.rule.id == rule_a_id).expect("Rule A missing");
        assert_eq!(res_a.rule.name, "Rule A");
        assert_eq!(res_a.assignments.len(), 2);

        let res_b = results.iter().find(|r| r.rule.id == rule_b_id).expect("Rule B missing");
        assert_eq!(res_b.assignments.len(), 1);

        assert!(results.iter().find(|r| r.rule.id == rule_c_id).is_none());
    }

    #[tokio::test]
    async fn test_fetch_status_range() {
        // [Setup]
        let pool = setup_test_db().await;
        let repo = CalendarRepository::new(pool.clone());

        let plan_id = create_plan(&pool, "Plan").await;
        let cal_id = create_calendar(&pool, plan_id).await;
        let rule_id = create_rule(&pool, plan_id, "Rule").await;

        // DBにデータを投入 (0〜4週目)
        create_status(&pool, cal_id, 0, "Active", Some(10), Some(rule_id)).await;
        create_status(&pool, cal_id, 1, "Skipped", None, None).await;
        create_status(&pool, cal_id, 2, "Active", Some(11), Some(rule_id)).await;
        create_status(&pool, cal_id, 3, "Skipped", None, None).await;
        create_status(&pool, cal_id, 4, "Active", Some(12), Some(rule_id)).await;

        // [Act] Offset 1 から 3つ分 (1, 2, 3) を取得
        let results = repo.fetch_status_range(cal_id, 1, 3).await.expect("Failed to fetch range");

        // [Assert]
        assert_eq!(results.len(), 3);
        assert!(matches!(results[0], WeekStatus::Skipped));
        match &results[1] {
            WeekStatus::Active { logical_delta, rule_id: r_id } => {
                assert_eq!(*logical_delta, 11);
                assert_eq!(*r_id as i64, rule_id);
            },
            _ => panic!("Expected Active for offset 2"),
        }
        assert!(matches!(results[2], WeekStatus::Skipped));
    }
}
