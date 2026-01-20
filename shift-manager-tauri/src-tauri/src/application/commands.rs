use tauri::State;
use crate::domain::{
    rule_model::*,
    shift_calendar_model::*,
};


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

// --- Group / Member Commands ---

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

// --- Rule / Assignment Commands ---

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
pub async fn add_rule_assignment(
    rule_id: i64, 
    weekday: i64, 
    shift_time: i64, 
    group_id: i64, 
    member_index: i64,
    repo: State<'_, AppServices>
) -> Result<i64, String> {
    repo.rule.add_rule_assignment(rule_id, weekday, shift_time, group_id, member_index).await
}

#[tauri::command]
pub async fn delete_assignment(assignment_id: i64, repo: State<'_, AppServices>) -> Result<(), String> {
    repo.rule.delete_assignment(assignment_id).await
}
