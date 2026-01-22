#[cfg(test)]
mod calendar_repo_tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::{SqlitePool};

    use shift_manager_tauri_lib::{
        domain::{
            shift_calendar_model::{
                WeekStatus,
            }
        },
        infrastructure::calendar_repo::CalendarRepository,
    };

    // ========================================================================
    // 1. テスト用セットアップ (Migrate!使用版)
    // ========================================================================
    
    // 本番と同じマイグレーションを実行してDBを用意する
    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:") // インメモリDBを使用
            .await
            .expect("Failed to create memory pool");

        // ★重要: ./migrations フォルダのSQL定義を適用
        // これにより本番環境と完全に同じテーブル構造が保証されます
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    // ========================================================================
    // 2. ヘルパー関数 (テストデータ作成用)
    // ========================================================================

    async fn create_plan(pool: &SqlitePool, name: &str) -> i64 {
        sqlx::query("INSERT INTO plans (name) VALUES (?)")
            .bind(name)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

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

    async fn create_assignment(pool: &SqlitePool, rule_id: i64, weekday: i64, group_id: i64) {
        sqlx::query(
            "INSERT INTO rule_assignments 
            (weekly_rule_id, weekday, shift_time_type, target_group_id, target_member_index) 
            VALUES (?, ?, 0, ?, 0)"
        )
        .bind(rule_id)
        .bind(weekday)
        .bind(group_id)
        .execute(pool)
        .await
        .unwrap();
    }

    // カレンダー親テーブル作成 (初期状態)
    async fn create_calendar(pool: &SqlitePool, plan_id: i64) -> i64 {
        sqlx::query("INSERT INTO shift_calendars (plan_id, base_abs_week, initial_delta) VALUES (?, 100, 0)")
            .bind(plan_id)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    // カレンダーステータス手動挿入 (データ準備用)
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

    // ========================================================================
    // 3. テストケース
    // ========================================================================

    #[tokio::test]
    async fn test_fetch_rules_by_ids() {
        let pool = setup_test_db().await;
        let repo = CalendarRepository::new(pool.clone());

        let plan_id = create_plan(&pool, "Test Plan").await;
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
        create_assignment(&pool, rule_c_id, 6, group_id).await; // Sun (Fetch対象外にする用)

        // [Act] Rule A と B だけを取得
        let results = repo.fetch_rules_by_ids(&[rule_a_id, rule_b_id]).await.expect("Failed to fetch");

        // [Assert]
        assert_eq!(results.len(), 2);
        
        let res_a = results.iter().find(|r| r.rule.id == rule_a_id).expect("Rule A missing");
        assert_eq!(res_a.rule.name, "Rule A");
        assert_eq!(res_a.assignments.len(), 2);

        let res_b = results.iter().find(|r| r.rule.id == rule_b_id).expect("Rule B missing");
        assert_eq!(res_b.assignments.len(), 1);

        // Rule C は取得されていないこと
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
        
        // Offset 1: Skipped
        assert!(matches!(results[0], WeekStatus::Skipped));
        
        // Offset 2: Active
        match &results[1] {
            WeekStatus::Active { logical_delta, rule_id: r_id } => {
                assert_eq!(*logical_delta, 11);
                assert_eq!(*r_id as i64, rule_id);
            },
            _ => panic!("Expected Active for offset 2"),
        }
        
        // Offset 3: Skipped
        assert!(matches!(results[2], WeekStatus::Skipped));
    }

    #[tokio::test]
    async fn test_save_timeline_append_logic() {
        // [Setup]
        let pool = setup_test_db().await;
        let repo = CalendarRepository::new(pool.clone());

        // ★修正1: create_plan ヘルパーを使って、実在するPlanを作成する
        let plan_id = create_plan(&pool, "Test Plan").await;

        // ★修正2: create_rule ヘルパーを使って、実在するRuleを作成する
        let rule_a = create_rule(&pool, plan_id, "Rule A").await;
        let rule_b = create_rule(&pool, plan_id, "Rule B").await;
        let rule_c = create_rule(&pool, plan_id, "Rule C").await;

        // テスト用のタイムラインデータ
        // マジックナンバー(101, 102)ではなく、作成した rule_a, rule_b のIDを使う
        let initial_timeline = vec![
            WeekStatus::Active { logical_delta: 0, rule_id: rule_a }, // rule_a
            WeekStatus::Skipped,
            WeekStatus::Active { logical_delta: 1, rule_id: rule_b }, // rule_b
        ];

        // -------------------------------------------------------
        // Case 1: 初回保存
        // -------------------------------------------------------
        repo.save_timeline(plan_id, 100, 0, &initial_timeline)
            .await
            .expect("First save failed");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weekly_statuses")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 3, "Should have 3 records initially");

        // -------------------------------------------------------
        // Case 2: 重複データの保存
        // -------------------------------------------------------
        repo.save_timeline(plan_id, 100, 0, &initial_timeline)
            .await
            .expect("Second save failed");

        let count_after_dup: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weekly_statuses")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count_after_dup, 3, "Count should ensure idempotency");

        // -------------------------------------------------------
        // Case 3: 追記 (Append New Data)
        // -------------------------------------------------------
        let mut extended_timeline = initial_timeline.clone();
        // 新規分を追加 (ここでも実在するルールIDを使う)
        extended_timeline.push(WeekStatus::Skipped);
        extended_timeline.push(WeekStatus::Active { logical_delta: 2, rule_id: rule_c }); // rule_c

        repo.save_timeline(plan_id, 100, 0, &extended_timeline)
            .await
            .expect("Append save failed");

        let count_final: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weekly_statuses")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count_final, 5, "Should verify 2 new records appended");

        // 最後のデータ確認
        let last_row: (i64, String) = sqlx::query_as("SELECT week_offset, status_type FROM weekly_statuses WHERE week_offset = 4")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(last_row.0, 4);
        assert_eq!(last_row.1, "Active");
    }
}
