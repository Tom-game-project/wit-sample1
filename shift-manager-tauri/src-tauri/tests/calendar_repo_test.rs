#[cfg(test)]
mod calendar_repo_tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::{SqlitePool};

    use shift_manager_tauri_lib::{
        domain::{
            shift_calendar_model::{WeekStatus},
        },
        // RuleRepository をインポート
        infrastructure::{
            calendar_repo::CalendarRepository,
            rule_repo::RuleRepository,
        },
    };

    // ========================================================================
    // 1. テスト用セットアップ
    // ========================================================================
    
    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create memory pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    // ========================================================================
    // 2. ヘルパー関数 (RuleRepositoryでカバーできない部分のみ残す)
    // ========================================================================
    // Plan, Rule, Group, Assignment の作成は RuleRepository に任せるため削除しました。

    // CalendarRepositoryのテスト用データ投入のために、
    // "Repositoryを使わずに直接DBを操作したい" ケース（手動セットアップ）のみヘルパーを残します。

    async fn create_calendar_manual(pool: &SqlitePool, plan_id: i64) -> i64 {
        sqlx::query("INSERT INTO shift_calendars (plan_id, base_abs_week, initial_delta) VALUES (?, 100, 0)")
            .bind(plan_id)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn create_status_manual(pool: &SqlitePool, cal_id: i64, offset: i64, status: &str, delta: Option<i64>, rule_id: Option<i64>) {
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
        // 両方のリポジトリをインスタンス化
        let cal_repo = CalendarRepository::new(pool.clone());
        let rule_repo = RuleRepository::new(pool.clone());

        // [Setup] RuleRepository を使ってデータを構築
        let plan_id = rule_repo.create_plan("Test Plan").await.expect("Failed to create plan");
        let group_id = rule_repo.add_staff_group(plan_id, "Group A").await.expect("Failed to create group");

        // Rule A
        let rule_a_id = rule_repo.add_weekly_rule(plan_id, "Rule A").await.expect("Failed to create rule");
        rule_repo.add_rule_assignment(rule_a_id, 0, 0, group_id, 0).await.expect("Failed to assign"); // Mon
        rule_repo.add_rule_assignment(rule_a_id, 1, 0, group_id, 0).await.expect("Failed to assign"); // Tue

        // Rule B
        let rule_b_id = rule_repo.add_weekly_rule(plan_id, "Rule B").await.expect("Failed to create rule");
        rule_repo.add_rule_assignment(rule_b_id, 5, 0, group_id, 0).await.expect("Failed to assign"); // Sat

        // Rule C
        let rule_c_id = rule_repo.add_weekly_rule(plan_id, "Rule C").await.expect("Failed to create rule");
        rule_repo.add_rule_assignment(rule_c_id, 6, 0, group_id, 0).await.expect("Failed to assign"); 

        // [Act] Rule A と B だけを取得
        let results = cal_repo.fetch_rules_by_ids(&[rule_a_id, rule_b_id]).await.expect("Failed to fetch");

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
        let cal_repo = CalendarRepository::new(pool.clone());
        let rule_repo = RuleRepository::new(pool.clone());

        // RuleRepositoryで基本データ作成
        let plan_id = rule_repo.create_plan("Plan").await.unwrap();
        let rule_id = rule_repo.add_weekly_rule(plan_id, "Rule").await.unwrap();

        // Calendarのデータ投入は、CalendarRepoの保存機能テストではないため、
        // ここでは「手動ヘルパー」を使って、特定の状態(Active/Skippedの並び)を強制的に作ります。
        let cal_id = create_calendar_manual(&pool, plan_id).await;

        create_status_manual(&pool, cal_id, 0, "Active", Some(10), Some(rule_id)).await;
        create_status_manual(&pool, cal_id, 1, "Skipped", None, None).await;
        create_status_manual(&pool, cal_id, 2, "Active", Some(11), Some(rule_id)).await;
        create_status_manual(&pool, cal_id, 3, "Skipped", None, None).await;
        create_status_manual(&pool, cal_id, 4, "Active", Some(12), Some(rule_id)).await;

        // [Act] Offset 1 から 3つ分 (1, 2, 3) を取得
        let results = cal_repo.fetch_status_range(cal_id, 1, 3).await.expect("Failed to fetch range");

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

    #[tokio::test]
    async fn test_save_timeline_append_logic() {
        // [Setup]
        let pool = setup_test_db().await; 
        let cal_repo = CalendarRepository::new(pool.clone());
        let rule_repo = RuleRepository::new(pool.clone());

        // ★ RuleRepository を使用してプランとルールを作成
        // これにより、実際のアプリと同じ手順でFK制約を満たすデータが作られます
        let plan_id = rule_repo.create_plan("Test Plan").await.expect("Failed to create plan");
        let rule_a_id = rule_repo.add_weekly_rule(plan_id, "Rule A").await.expect("Failed rule A");
        let rule_b_id = rule_repo.add_weekly_rule(plan_id, "Rule B").await.expect("Failed rule B");
        let rule_c_id = rule_repo.add_weekly_rule(plan_id, "Rule C").await.expect("Failed rule C");

        // テスト用のタイムラインデータ
        let initial_timeline = vec![
            WeekStatus::Active { logical_delta: 0, rule_id: rule_a_id }, 
            WeekStatus::Skipped,
            WeekStatus::Active { logical_delta: 1, rule_id: rule_b_id },
        ];

        // -------------------------------------------------------
        // Case 1: 初回保存
        // -------------------------------------------------------
        cal_repo.save_timeline(plan_id, 100, 0, &initial_timeline)
            .await
            .expect("First save failed");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weekly_statuses")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 3, "Should have 3 records initially");

        // -------------------------------------------------------
        // Case 2: 重複データの保存 (Idempotency Check)
        // -------------------------------------------------------
        cal_repo.save_timeline(plan_id, 100, 0, &initial_timeline)
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
        extended_timeline.push(WeekStatus::Skipped);                                     // index 3
        extended_timeline.push(WeekStatus::Active { logical_delta: 2, rule_id: rule_c_id }); // index 4

        cal_repo.save_timeline(plan_id, 100, 0, &extended_timeline)
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
