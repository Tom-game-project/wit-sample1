// =====================
// ドメインモデル定義
// =====================

use serde::{Serialize, Deserialize};
use shift_calendar::shift_gen;

// 絶対週番号と論理デルタの型エイリアス
pub type AbsWeek = usize;
pub type LogicalDelta = usize;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum WeekStatus {
    Active { logical_delta: LogicalDelta },
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
    
    pub base_abs_week: AbsWeek,
    pub initial_delta: LogicalDelta,
    pub timeline: Vec<WeekStatus>,
}
