use serde::{Serialize};
//
// Rules
//

use sqlx::{FromRow, prelude::Type};

// --- 1. Plan (設定セット/親) ---
#[derive(Debug, Serialize, FromRow)]
pub struct Plan {
    pub id: i64,
    pub name: String,
    // created_at はRust側で扱わないなら省略可
}

// --- 2. Staff Group ---
#[derive(Debug, Serialize, FromRow)]
pub struct StaffGroup {
    pub id: i64,
    pub plan_id: i64,
    pub name: String,
    pub sort_order: i64,
}

// --- 3. Staff Member ---
#[derive(Debug, Serialize, FromRow)]
pub struct StaffMember {
    pub id: i64,
    pub group_id: i64,
    pub name: String,
    pub sort_order: i64,
}

// --- 4. Weekly Rule ---
#[derive(Debug, Serialize, FromRow)]
pub struct WeeklyRule {
    pub id: i64,
    pub plan_id: i64,
    pub name: String,
    pub sort_order: i64,
}

/// 曜日を表す Enum (DBの 0~6 と自動マッピング)
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Type)]
#[repr(i64)] // DBの INTEGER(i64) として扱う指定
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

/// シフト時間帯を表す Enum (DBの 0~1 と自動マッピング)
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Type)]
#[repr(i64)]
pub enum ShiftTime {
    Morning = 0,
    Afternoon = 1,
}

// --- 5. Rule Assignment (Holl) ---
// #[derive(Debug, Serialize, FromRow, Clone)]
// // #[serde(rename_all = "camelCase")] // JS側は camelCase が一般的
// pub struct RuleAssignment {
//     pub id: i64,
//     pub weekly_rule_id: i64,
//     pub weekday: i64,         // 0:Mon - 6:Sun
//     pub shift_time_type: i64, // 0:Morning, 1:Afternoon
//     pub target_group_id: i64,
//     pub target_member_index: i64, // sort_orderに対応
// }

/// 改善されたルールアサイン構造体
#[derive(Debug, Serialize, FromRow, Clone)]
pub struct RuleAssignment {
    pub id: i64,
    pub weekly_rule_id: i64,

    pub weekday: Weekday,
    pub shift_time_type: ShiftTime,

    pub target_group_id: i64,

    // インデックスや順序を表す場合は usize にしておくのがRustの定石
    #[sqlx(try_from = "i64")]
    pub target_member_index: usize, 
}

// --- 複合データ (フロントエンドに一括で返す用) ---
// Planを選択したときに、紐づく設定を全部まとめて返すための構造体
#[derive(Debug, Serialize)]
// #[serde(rename_all = "camelCase")]
pub struct PlanConfig {
    pub plan: Plan,
    pub groups: Vec<StaffGroupWithMembers>,
    pub rules: Vec<WeeklyRuleWithAssignments>,
}

#[derive(Debug, Serialize)]
pub struct StaffGroupWithMembers {
    pub group: StaffGroup,
    pub members: Vec<StaffMember>,
}

#[derive(Debug, Serialize)]
pub struct WeeklyRuleWithAssignments {
    pub rule: WeeklyRule,
    pub assignments: Vec<RuleAssignment>,
}
