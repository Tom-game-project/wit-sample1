use serde::{Deserialize, Serialize};

// ==========================================
// 1. スタッフグループ定義
// ==========================================

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonSlot {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonStaffGroup {
    pub name: String,
    pub slots: Vec<JsonSlot>,
}

// ==========================================
// 2. ルール・スケジュール定義
// ==========================================

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")] // JSONの camelCase を Rustの snake_case に対応させる
pub struct JsonAssignment {
    pub staff_group_id: u32,
    pub shift_staff_index: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonDailySchedule {
    pub m: Vec<JsonAssignment>, // 午前 (Morning)
    pub a: Vec<JsonAssignment>, // 午後 (Afternoon)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonWeeklySchedule {
    pub mon: JsonDailySchedule,
    pub tue: JsonDailySchedule,
    pub wed: JsonDailySchedule,
    pub thu: JsonDailySchedule,
    pub fri: JsonDailySchedule,
    pub sat: JsonDailySchedule,
    pub sun: JsonDailySchedule,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRule {
    pub name: String,
    pub schedule: JsonWeeklySchedule,
}

// ==========================================
// 3. ルート定義 (全体)
// ==========================================

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonConfig {
    pub staff_groups: Vec<JsonStaffGroup>,
    pub rules: Vec<JsonRule>,
}

/*
pub fn load_config_from_json(, json_str: &str) -> Result<(), String> {
    // 1. JSON文字列を Rustの構造体にパース
    let config: JsonConfig = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    // 2. 内部状態をクリア (必要に応じて)
    self.staff_groups.clear();
    self.rules.clear();

    // 3. データを内部構造に移し替える
    // (JsonConfigの構造と内部構造が完全に一致しているならそのまま代入でOKですが、
    //  型が違う場合はここで変換します)

    // --- Staff Groups のロード ---
    for group in config.staff_groups {
        // 内部用のStaffGroup型に変換して追加
        self.add_new_group_with_data(group.name, group.slots);
    }

    // --- Rules のロード ---
    for rule in config.rules {
        // 内部用のRule型に変換して追加
        self.add_rule_with_data(rule.name, rule.schedule);
    }

    Ok(())
}
*/
