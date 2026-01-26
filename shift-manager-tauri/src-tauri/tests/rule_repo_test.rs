#[cfg(test)]
mod rule_repo_tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use shift_manager_tauri_lib::infrastructure::rule_repo::*;

    // 1. テスト用DBセットアップ (最新スキーマ反映)
    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create memory pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to create schema");

        pool
    }

    // 2. 統合テスト: Configの保存と復元
    #[tokio::test]
    async fn test_create_and_fetch_full_config() {
        let pool = setup_test_db().await;
        let repo = RuleRepository::new(pool);

        // A. Plan作成
        let plan_id = repo.create_plan("Test Plan 2026").await.unwrap();

        // B. Group & Member作成
        let group_id = repo.add_staff_group(plan_id, "Kitchen").await.unwrap();
        let _member1_id = repo.add_staff_member(group_id, "Tanaka").await.unwrap();
        let _member2_id = repo.add_staff_member(group_id, "Suzuki").await.unwrap();

        // C. Rule & Assignment作成
        let rule_id = repo.add_weekly_rule(plan_id, "Basic Week").await.unwrap();

        // 注意: Assignmentは target_group_id が必要
        // ここで作成した group_id を指定することで外部キー制約を満たす
        repo.add_rule_assignment(rule_id, 0, 0, group_id, 0).await.unwrap(); // Mon, Morning, Kitchen:0(Tanaka)
        repo.add_rule_assignment(rule_id, 0, 1, group_id, 1).await.unwrap(); // Mon, Afternoon, Kitchen:1(Suzuki)

        // D. 一括取得 (get_plan_config)
        let config = repo.get_plan_config(plan_id).await.unwrap();

        // E. 検証
        assert_eq!(config.plan.name, "Test Plan 2026");

        // Groupsチェック
        assert_eq!(config.groups.len(), 1);
        assert_eq!(config.groups[0].group.name, "Kitchen");
        assert_eq!(config.groups[0].members.len(), 2);
        assert_eq!(config.groups[0].members[0].name, "Tanaka");

        // Rulesチェック
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].rule.name, "Basic Week");
        assert_eq!(config.rules[0].assignments.len(), 2);
    }

    // 3. テスト: Cascade Deleteの確認
    #[tokio::test]
    async fn test_cascade_delete() {
        let pool = setup_test_db().await;
        let repo = RuleRepository::new(pool.clone());

        // データ作成
        let plan_id = repo.create_plan("Delete Me").await.unwrap();
        let group_id = repo.add_staff_group(plan_id, "Group").await.unwrap();
        let _rule_id = repo.add_weekly_rule(plan_id, "Rule").await.unwrap();

        // Plan削除
        repo.delete_plan(plan_id).await.unwrap();

        // 検証: 子データも消えているはず
        let group_exists: Option<i64> = sqlx::query_scalar("SELECT id FROM staff_groups WHERE id = ?")
            .bind(group_id)
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(group_exists.is_none(), "Plan削除に伴いGroupも削除されているべき");
    }
}
