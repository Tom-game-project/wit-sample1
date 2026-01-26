use std::collections::HashMap;

use tauri::State;
use crate::application::time::{calculate_abs_week, calculate_weeks_in_month};
use crate::domain::calendar_logic::calculate_partial_shift;
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
pub async fn get_calendar_state(plan_id: i64, repo: State<'_, AppServices>) -> Result<Option<ShiftCalendarManager>, String> {
    repo.calendar.find_by_plan_id(plan_id).await
}

// --- Calendar ---
#[tauri::command]
pub async fn create_calendar(plan_id: i64, base_abs_week: usize, initial_delta: usize, repo: State<'_, AppServices>) -> Result<i64, String> {
    // Repository側の create_calendar を呼び出す
    repo.calendar.create_calendar(plan_id, base_abs_week, initial_delta).await
}

#[tauri::command]
pub async fn append_timeline(plan_id: i64, start_abs_week: usize, statuses: Vec<Option<i64>>, repo: State<'_, AppServices>) -> Result<(), String> {
    // Repository側の try_to_append_timeline を呼び出す
    repo.calendar.try_to_append_timeline(plan_id, start_abs_week, statuses).await
}

use crate::application::dto::{MonthlyShiftResult, WeeklyShiftDto, DailyShiftDto};

use shift_calendar::shift_gen::{DayRule, Incomplete, ShiftHoll, StaffGroup, StaffGroupList, WeekRule, WeekRuleTable};

/// ====================================================================
/// 1. StaffGroupList の構築と、IDマップの作成
/// ====================================================================
// 戻り値をタプルにし、マップも一緒に返すように変更
fn db2staff_group_domain(plan_config: &PlanConfig) -> (StaffGroupList, HashMap<i64, usize>) {
    let mut domain_groups = StaffGroupList::new();
    let mut group_id_map = HashMap::new(); // ★追加: DBのIDとインデックスの対応表

    for group_row in &plan_config.groups {
        let mut domain_members = StaffGroup::new(&group_row.group.name);
        for member_row in &group_row.members {
            // DBのメンバー情報をドメインの Staff 型に変換
            domain_members.add_staff(&member_row.name);
        }

        // ★追加: DBのグループIDが、これから何番目(インデックス)に入るかを記憶する
        let current_index = domain_groups.0.len();
        group_id_map.insert(group_row.group.id, current_index);

        domain_groups.add_staff_group(domain_members);
    }
    return (domain_groups, group_id_map); // 両方を返す
}

/// ====================================================================
/// 2. ルール辞書 (HashMap) の構築
/// ====================================================================
// 引数に group_id_map を追加
fn db2rule_domain<'a>(
    plan_config: &PlanConfig,
    group_id_map: &HashMap<i64, usize>
) -> HashMap<i64, WeekRuleTable<'a, Incomplete>> {
    let mut rule_dict: HashMap<i64, WeekRuleTable<'_, Incomplete>> = HashMap::new();

    for rule_row in &plan_config.rules {
        let rule_id = rule_row.rule.id;
        let mut week_table = WeekRuleTable::new();

        let mut days:[DayRule<'_, Incomplete>; 7] = core::array::from_fn(|_| DayRule {
            shift_morning: Vec::new(),
            shift_afternoon: Vec::new(),
        });

        for assign in &rule_row.assignments {
            let week_day = assign.weekday;
            let shift_time = assign.shift_time_type;

            let day = match week_day {
                Weekday::Monday    => &mut days[0],
                Weekday::Tuesday   => &mut days[1],
                Weekday::Wednesday => &mut days[2],
                Weekday::Thursday  => &mut days[3],
                Weekday::Friday    => &mut days[4],
                Weekday::Saturday  => &mut days[5],
                Weekday::Sunday    => &mut days[6],
            };

            // ★修正: 危険な `- 1` をやめ、マップから安全にインデックスを取得する
            let group_index = *group_id_map.get(&assign.target_group_id)
                .expect("DBの整合性エラー：存在しないグループIDがアサインされています");

            match shift_time {
                ShiftTime::Morning =>
                    day.shift_morning.push(
                        ShiftHoll::new(
                            group_index, // ★取得した安全なインデックスを使う
                            assign.target_member_index as usize,
                        )
                    ),
                ShiftTime::Afternoon =>
                    day.shift_afternoon.push(
                        ShiftHoll::new(
                            group_index, // ★ここも同じ
                            assign.target_member_index as usize,
                        )
                    ),
            }
        }
        week_table.add_week_rule(WeekRule(days));
        rule_dict.insert(rule_id, week_table);
    }
    rule_dict
}

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

    let calendar = match manager_opt {
        Some(m) => m,
        None => return Ok(MonthlyShiftResult { weeks: vec![] }), // データなし
    };

    // ★ ここでカレンダーIDと基準週を取り出します
    let calendar_id = if let Some(cal_id) = calendar.id {
        cal_id
    } else {
        return Err(String::from("カレンダーを作成してください"))
    };

    let base_abs_week = calendar.base_abs_week as usize; // 型がusizeの場合はキャスト

    // 2. 計算に必要な「辞書データ」をDBから全取得して構築
    //    (本来はRepositoryにこの変換ロジックを持たせるのが綺麗ですが、ここでやります)
    let plan_config = repo.rule.get_plan_config(plan_id).await?;

    let start_week_abs = if let Some (week_abs) = calculate_abs_week(target_year, target_month, 1) {
        week_abs
    } else {
        return Err(String::from("base abs の計算に失敗しました"));
    };

    let range = calculate_weeks_in_month(target_year, target_month); // カレンダーは最大6週表示

    let start_offset = start_week_abs - base_abs_week;

    let week_status_list = repo.calendar.fetch_status_range(
        calendar_id,
        start_offset as i64,
        range as i64).await?;

    // databaseをドメインロジック向けに編集する

    // 1. DBからドメインへの変換と、IDマップの取得
    let (domain_groups, group_id_map) = db2staff_group_domain(&plan_config); 

    // 2. マップを使ってルールを変換
    let rule_dict = db2rule_domain(&plan_config, &group_id_map);

    // 3. コアロジック実行
    let partial_shift = calculate_partial_shift(&week_status_list, &rule_dict, &domain_groups);

    let dto_weeks: Vec<Option<WeeklyShiftDto>> = partial_shift
        .into_iter()
        .map(|week_opt| {
            // 週データが存在する(Some)場合だけ、中身を変換する
            week_opt.map(|week| {

                // 1週間分(7日)のデータをループして DailyShiftDto の Vec を作る
                let days_dto: Vec<DailyShiftDto> = week.0
                    .into_iter()
                    .map(|day| DailyShiftDto {
                        morning: day.shift_morning.iter().map(|t| t.name.clone()).collect(),
                        afternoon: day.shift_afternoon.iter().map(|t| t.name.clone()).collect(),
                    })
                    .collect();

                // WeeklyShiftDto に詰める
                WeeklyShiftDto { days: days_dto }
            })
        })
        .collect();

    Ok(MonthlyShiftResult { weeks: dto_weeks })
}


