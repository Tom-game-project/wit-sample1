use serde::Serialize;

/// 1日分の確定シフト (フロントエンド表示用)
#[derive(Debug, Serialize)]
pub struct DailyShiftDto {
    pub morning: Vec<String>,   // 午前のアサイン名リスト (例: ["Tanaka", "Suzuki"])
    pub afternoon: Vec<String>, // 午後のアサイン名リスト
}

/// 1週間分の確定シフト
#[derive(Debug, Serialize)]
pub struct WeeklyShiftDto {
    pub days: Vec<DailyShiftDto>, // 0(Mon) ~ 6(Sun)
}

/// コマンドの返り値
#[derive(Debug, Serialize)]
pub struct MonthlyShiftResult {
    // フロントエンドのカレンダー週順 (0, 1, 2...) に対応するデータ
    // None = その週はスキップ、または未生成
    pub weeks: Vec<Option<WeeklyShiftDto>>,
}
