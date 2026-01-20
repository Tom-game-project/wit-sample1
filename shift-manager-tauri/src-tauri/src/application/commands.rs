use tauri::State;
use crate::infrastructure::repository::CalendarRepository;
use crate::domain::{
    rule_model::*,
    shift_calendar_model::*,
};

// --- Plan Commands ---

#[tauri::command]
pub async fn create_new_plan(name: String, repo: State<'_, CalendarRepository>) -> Result<i64, String> {
    repo.create_plan(&name).await
}

#[tauri::command]
pub async fn list_all_plans(repo: State<'_, CalendarRepository>) -> Result<Vec<Plan>, String> {
    repo.list_plans().await
}

#[tauri::command]
pub async fn delete_plan(id: i64, repo: State<'_, CalendarRepository>) -> Result<(), String> {
    repo.delete_plan(id).await
}

#[tauri::command]
pub async fn get_plan_config(plan_id: i64, repo: State<'_, CalendarRepository>) -> Result<PlanConfig, String> {
    repo.get_plan_config(plan_id).await
}

// --- Group / Member Commands ---

#[tauri::command]
pub async fn add_staff_group(plan_id: i64, name: String, repo: State<'_, CalendarRepository>) -> Result<i64, String> {
    repo.add_staff_group(plan_id, &name).await
}

#[tauri::command]
pub async fn delete_staff_group(group_id: i64, repo: State<'_, CalendarRepository>) -> Result<(), String> {
    repo.delete_staff_group(group_id).await
}

#[tauri::command]
pub async fn update_group_name(group_id: i64, name: String, repo: State<'_, CalendarRepository>) -> Result<(), String> {
    repo.update_group_name(group_id, &name).await
}

#[tauri::command]
pub async fn add_staff_member(group_id: i64, name: String, repo: State<'_, CalendarRepository>) -> Result<i64, String> {
    repo.add_staff_member(group_id, &name).await
}

#[tauri::command]
pub async fn delete_staff_member(member_id: i64, repo: State<'_, CalendarRepository>) -> Result<(), String> {
    repo.delete_staff_member(member_id).await
}

#[tauri::command]
pub async fn update_member_name(member_id: i64, name: String, repo: State<'_, CalendarRepository>) -> Result<(), String> {
    repo.update_member_name(member_id, &name).await
}

// --- Rule / Assignment Commands ---

#[tauri::command]
pub async fn add_weekly_rule(plan_id: i64, name: String, repo: State<'_, CalendarRepository>) -> Result<i64, String> {
    repo.add_weekly_rule(plan_id, &name).await
}

#[tauri::command]
pub async fn delete_weekly_rule(rule_id: i64, repo: State<'_, CalendarRepository>) -> Result<(), String> {
    repo.delete_weekly_rule(rule_id).await
}

#[tauri::command]
pub async fn update_rule_name(rule_id: i64, name: String, repo: State<'_, CalendarRepository>) -> Result<(), String> {
    repo.update_rule_name(rule_id, &name).await
}

#[tauri::command]
pub async fn add_rule_assignment(
    rule_id: i64, 
    weekday: i64, 
    shift_time: i64, 
    group_id: i64, 
    member_index: i64,
    repo: State<'_, CalendarRepository>
) -> Result<i64, String> {
    repo.add_rule_assignment(rule_id, weekday, shift_time, group_id, member_index).await
}

#[tauri::command]
pub async fn delete_assignment(assignment_id: i64, repo: State<'_, CalendarRepository>) -> Result<(), String> {
    repo.delete_assignment(assignment_id).await
}
