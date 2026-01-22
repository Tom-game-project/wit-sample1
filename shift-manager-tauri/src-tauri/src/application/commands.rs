use tauri::State;
use crate::domain::{rule_model::*, shift_calendar_model::*};
use crate::AppServices;

// --- Plan Commands ---
#[tauri::command]
pub async fn create_new_plan(name: String, repo: State<'_, AppServices>) -> Result<i64, String> {
    repo.rule.create_plan(&name).await
}

#[tauri::command]
pub async fn list_all_plans(repo: State<'_, AppServices>) -> Result<Vec<Plan>, String> {
    repo.rule.list_plans().await
}

#[tauri::command]
pub async fn delete_plan(id: i64, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.delete_plan(id).await
}

#[tauri::command]
pub async fn get_plan_config(plan_id: i64, repo: State<'_, AppServices>) -> Result<PlanConfig, String> {
    repo.rule.get_plan_config(plan_id).await
}

// --- Group / Member ---
#[tauri::command]
pub async fn add_staff_group(plan_id: i64, name: String, repo: State<'_, AppServices>) -> Result<i64, String> {
    repo.rule.add_staff_group(plan_id, &name).await
}

#[tauri::command]
pub async fn delete_staff_group(group_id: i64, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.delete_staff_group(group_id).await
}

#[tauri::command]
pub async fn update_group_name(group_id: i64, name: String, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.update_group_name(group_id, &name).await
}

#[tauri::command]
pub async fn add_staff_member(group_id: i64, name: String, repo: State<'_, AppServices>) -> Result<i64, String> {
    repo.rule.add_staff_member(group_id, &name).await
}

#[tauri::command]
pub async fn delete_staff_member(member_id: i64, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.delete_staff_member(member_id).await
}

#[tauri::command]
pub async fn update_member_name(member_id: i64, name: String, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.update_member_name(member_id, &name).await
}

// --- Rules ---
#[tauri::command]
pub async fn add_weekly_rule(plan_id: i64, name: String, repo: State<'_, AppServices>) -> Result<i64, String> {
    repo.rule.add_weekly_rule(plan_id, &name).await
}

#[tauri::command]
pub async fn delete_weekly_rule(rule_id: i64, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.delete_weekly_rule(rule_id).await
}

#[tauri::command]
pub async fn update_rule_name(rule_id: i64, name: String, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.update_rule_name(rule_id, &name).await
}

#[tauri::command]
pub async fn add_rule_assignment(rule_id: i64, weekday: i64, shift_time: i64, group_id: i64, member_index: i64, repo: State<'_, AppServices>) -> Result<i64, String> {
    repo.rule.add_rule_assignment(rule_id, weekday, shift_time, group_id, member_index).await
}

#[tauri::command]
pub async fn delete_assignment(assignment_id: i64, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.delete_assignment(assignment_id).await
}

// --- Calendar ---
#[tauri::command]
pub async fn save_calendar_state(manager: ShiftCalendarManager, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.calendar.save_calendar(&manager).await
}

#[tauri::command]
pub async fn get_calendar_state(plan_id: i64, repo: State<'_, AppServices>) -> Result<Option<ShiftCalendarManager>, String> {
    repo.calendar.find_by_plan_id(plan_id).await
}

use std::collections::HashMap;
use crate::application::dto::{MonthlyShiftResult, WeeklyShiftDto, DailyShiftDto};

/// 週ごとのシフト導出計算をします
#[tauri::command]
pub async fn derive_monthly_shift(
    plan_id: i64,
    target_year: i32,
    target_month: u32, // 0-11
    repo: State<'_, AppServices>,
) -> Result<MonthlyShiftResult, String> {
    // 1. カレンダーManager（タイムライン）を取得
    let manager_opt = repo.calendar.find_by_plan_id(plan_id).await?;

    // let manager = match manager_opt {
    //     Some(m) => m,
    //     None => return Ok(MonthlyShiftResult { weeks: vec![] }), // データなし
    // };

    // 2. 計算に必要な「辞書データ」をDBから全取得して構築
    //    (本来はRepositoryにこの変換ロジックを持たせるのが綺麗ですが、ここでやります)
    let plan_config = repo.rule.get_plan_config(plan_id).await?;

    // --- ドメインオブジェクトへの変換 (簡易実装) ---
    // ※ ユーザー様のドメイン型定義に合わせて調整が必要です
    // ここでは概念的な変換を示します

    // A. StaffGroupList の構築
    // let staff_group_list = StaffGroupList::from_config(&plan_config);
    // といった変換が必要です。ここではロジックに渡せる形になっていると仮定します。
    // (実際にはルールロジック内でID解決するために必要)

    // B. RuleMap の構築
    // let rule_map: HashMap<RuleId, WeekRuleTable> = ...;
    // plan_config.rules を回して、ドメインモデルの WeekRuleTable に変換します。


    // 3. 計算対象の期間を特定
    //    JS側の calculateCalendarDates とロジックを合わせる必要があります。
    //    ここでは「該当月の1日が含まれる週」を特定し、そこから6週間分取得すると仮定します。

    // 簡易計算: (ターゲット年月 - 基準日) / 7日 で絶対週を出すロジックが必要
    // 今回はデモとして「managerの先頭から表示する」あるいは「start_week_abs」を計算して渡します。
    // let start_week_abs =  calculate_abs_week(target_year, target_month, manager.base_abs_week);
    let start_week_abs = 1;
    let range = 6; // カレンダーは最大6週表示

    // 4. ロジック実行！ (ユーザー様の自作関数)
    // ※ ここで rule_map, staff_group_list を渡す
    // let domain_result = manager.derive_shift(
    //    &rule_map,
    //    &staff_group_list,
    //    start_week_abs,
    //    range
    // );

    // 5. 結果を DTO に変換 (ここが重要)
    //    ドメイン層の型を、JSON用の型(Stringのリスト)に焼き直します
    // let mut result_weeks = Vec::new();

    // for domain_week in domain_result {
    //     match domain_week {
    //         Some(week) => {
    //             let mut days_dto = Vec::new();
    //             for day_idx in 0..7 {
    //                  // weekの中から day_idx のシフトを取り出し、名前リストにする
    //                  let morning_names = week.get_assigned_names(day_idx, 0); // 仮定のメソッド
    //                  let afternoon_names = week.get_assigned_names(day_idx, 1);
    //                  days_dto.push(DailyShiftDto { morning: morning_names, afternoon: afternoon_names });
    //             }
    //             result_weeks.push(Some(WeeklyShiftDto { days: days_dto }));
    //         },
    //         None => result_weeks.push(None), // Skipまたは範囲外
    //     }
    // }

    // ★まだロジック結合前なので、ダミーデータを返してUIテストできるようにします
    // (これを実装すれば、UI側の準備が進められます)
    // ---------------------------------------------------
    let mut dummy_weeks = Vec::new();
    for _ in 0..6 {
        let mut days = Vec::new();
        for _ in 0..7 {
            days.push(DailyShiftDto {
                morning: vec!["Staff A".to_string()],
                afternoon: vec!["Staff B".to_string(), "Staff C".to_string()],
            });
        }
        dummy_weeks.push(Some(WeeklyShiftDto { days }));
    }
    // ---------------------------------------------------

    Ok(MonthlyShiftResult { weeks: dummy_weeks })
}

// ヘルパー: 年月から絶対週番号を計算する (JS側と合わせる必要あり)
fn calculate_abs_week(year: i32, month: u32, base_week: usize) -> usize {
    // TODO: 正確な日付計算の実装
    // 一旦、manager.base_abs_week をそのまま返す（常に先頭から表示）
    base_week
}
