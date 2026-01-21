use sqlx::SqlitePool;
use crate::domain::rule_model::*;

pub struct RuleRepository {
    pool: SqlitePool,
}

impl RuleRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // =================================================================
    // 1. Plan Operations (プラン操作)
    // =================================================================

    pub async fn create_plan(&self, name: &str) -> Result<i64, String> {
        let id = sqlx::query("INSERT INTO plans (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .last_insert_rowid();
        Ok(id)
    }

    pub async fn list_plans(&self) -> Result<Vec<Plan>, String> {
        sqlx::query_as::<_, Plan>("SELECT id, name FROM plans ORDER BY id DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_plan(&self, plan_id: i64) -> Result<(), String> {
        // ON DELETE CASCADE により、子要素も全削除される
        sqlx::query("DELETE FROM plans WHERE id = ?")
            .bind(plan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn update_plan_name(&self, plan_id: i64, name: &str) -> Result<(), String> {
        sqlx::query("UPDATE plans SET name = ? WHERE id = ?")
            .bind(name)
            .bind(plan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // =================================================================
    // 2. Staff Group & Member Operations
    // =================================================================

    pub async fn add_staff_group(&self, plan_id: i64, name: &str) -> Result<i64, String> {
        // 現在の最大sort_orderを取得して +1 する
        let next_order: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM staff_groups WHERE plan_id = ?"
        )
        .bind(plan_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let id = sqlx::query("INSERT INTO staff_groups (plan_id, name, sort_order) VALUES (?, ?, ?)")
            .bind(plan_id)
            .bind(name)
            .bind(next_order)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .last_insert_rowid();
        Ok(id)
    }

    pub async fn delete_staff_group(&self, group_id: i64) -> Result<(), String> {
        sqlx::query("DELETE FROM staff_groups WHERE id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn update_group_name(&self, group_id: i64, name: &str) -> Result<(), String> {
        sqlx::query("UPDATE staff_groups SET name = ? WHERE id = ?")
            .bind(name)
            .bind(group_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Member ---

    pub async fn add_staff_member(&self, group_id: i64, name: &str) -> Result<i64, String> {
        let next_order: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM staff_members WHERE group_id = ?"
        )
        .bind(group_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let id = sqlx::query("INSERT INTO staff_members (group_id, name, sort_order) VALUES (?, ?, ?)")
            .bind(group_id)
            .bind(name)
            .bind(next_order)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .last_insert_rowid();
        Ok(id)
    }

    pub async fn delete_staff_member(&self, member_id: i64) -> Result<(), String> {
        sqlx::query("DELETE FROM staff_members WHERE id = ?")
            .bind(member_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn update_member_name(&self, member_id: i64, name: &str) -> Result<(), String> {
        sqlx::query("UPDATE staff_members SET name = ? WHERE id = ?")
            .bind(name)
            .bind(member_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // =================================================================
    // 3. Weekly Rule & Assignment Operations
    // =================================================================

    pub async fn add_weekly_rule(&self, plan_id: i64, name: &str) -> Result<i64, String> {
        let next_order: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM weekly_rules WHERE plan_id = ?"
        )
        .bind(plan_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let id = sqlx::query("INSERT INTO weekly_rules (plan_id, name, sort_order) VALUES (?, ?, ?)")
            .bind(plan_id)
            .bind(name)
            .bind(next_order)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .last_insert_rowid();
        Ok(id)
    }

    pub async fn delete_weekly_rule(&self, rule_id: i64) -> Result<(), String> {
        sqlx::query("DELETE FROM weekly_rules WHERE id = ?")
            .bind(rule_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn update_rule_name(&self, rule_id: i64, name: &str) -> Result<(), String> {
        sqlx::query("UPDATE weekly_rules SET name = ? WHERE id = ?")
            .bind(name)
            .bind(rule_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Assignment ---

    pub async fn add_rule_assignment(
        &self,
        rule_id: i64,
        weekday: i64,
        shift_time: i64,
        group_id: i64,
        member_index: i64
    ) -> Result<i64, String> {
        let id = sqlx::query(
            "INSERT INTO rule_assignments (weekly_rule_id, weekday, shift_time_type, target_group_id, target_member_index)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(rule_id)
        .bind(weekday)
        .bind(shift_time)
        .bind(group_id)
        .bind(member_index)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();
        Ok(id)
    }

    pub async fn delete_assignment(&self, assignment_id: i64) -> Result<(), String> {
        sqlx::query("DELETE FROM rule_assignments WHERE id = ?")
            .bind(assignment_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // =================================================================
    // 4. Fetch Entire Config (一括取得)
    // =================================================================

    /// 特定のプランに紐づくすべての設定（グループ、メンバー、ルール、アサイン）を取得する
    /// フロントエンドの初期化や再描画に使用
    pub async fn get_plan_config(&self, plan_id: i64) -> Result<PlanConfig, String> {
        // 1. Plan
        let plan: Plan = sqlx::query_as("SELECT id, name FROM plans WHERE id = ?")
            .bind(plan_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Plan not found: {}", e))?;

        // 2. Groups
        let groups_rows: Vec<StaffGroup> = sqlx::query_as(
            "SELECT id, plan_id, name, sort_order FROM staff_groups WHERE plan_id = ? ORDER BY sort_order ASC"
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 3. Members (Loop Query - データ量が少なければこれで十分)
        let mut groups_with_members = Vec::new();
        for g in groups_rows {
            let members: Vec<StaffMember> = sqlx::query_as(
                "SELECT id, group_id, name, sort_order FROM staff_members WHERE group_id = ? ORDER BY sort_order ASC"
            )
            .bind(g.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            groups_with_members.push(StaffGroupWithMembers {
                group: g,
                members,
            });
        }

        // 4. Rules
        let rules_rows: Vec<WeeklyRule> = sqlx::query_as(
            "SELECT id, plan_id, name, sort_order FROM weekly_rules WHERE plan_id = ? ORDER BY sort_order ASC"
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 5. Assignments (Loop Query)
        let mut rules_with_assignments = Vec::new();
        for r in rules_rows {
            let assignments: Vec<RuleAssignment> = sqlx::query_as(
                "SELECT id, weekly_rule_id, weekday, shift_time_type, target_group_id, target_member_index
                 FROM rule_assignments WHERE weekly_rule_id = ?"
            )
            .bind(r.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            rules_with_assignments.push(WeeklyRuleWithAssignments {
                rule: r,
                assignments,
            });
        }

        Ok(PlanConfig {
            plan,
            groups: groups_with_members,
            rules: rules_with_assignments,
        })
    }
}

#[cfg(test)]
mod rule_repo_tests {
    use super::RuleRepository; // モジュールの位置に合わせて調整してください
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;

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
