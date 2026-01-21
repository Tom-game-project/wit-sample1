use serde::{Serialize, Deserialize};
// 絶対週番号と論理デルタの型エイリアス
pub type AbsWeek = usize;
pub type LogicalDelta = usize;
pub type RuleId = i64;
pub type PlanId = i64;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum WeekStatus {
    Active { logical_delta: LogicalDelta, rule_id: RuleId },
    Skipped,
}

/// シフトカレンダー管理者（メイン構造体）
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftCalendarManager {
    // DB保存時にIDが必要な場合に備えてOptionにしていますが、
    // 新規作成時はNone、読み込み時はSomeになります。
    // フロントエンドとのやり取りだけなら無くても構いません。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>, 
    
    pub plan_id: PlanId,
    pub base_abs_week: AbsWeek,
    pub initial_delta: LogicalDelta,
    pub timeline: Vec<WeekStatus>,
}



