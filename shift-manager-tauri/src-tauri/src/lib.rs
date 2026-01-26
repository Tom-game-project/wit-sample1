use sqlx::{ 
    sqlite::{
        SqlitePoolOptions,
        SqliteConnectOptions
    },
};

use std::fs;

use tauri::Manager;

pub mod domain;
pub mod infrastructure;
pub mod application;

use sqlx::SqlitePool;
use infrastructure::calendar_repo::CalendarRepository;
use infrastructure::rule_repo::RuleRepository;

// 全てのリポジトリを保持するコンテナ
pub struct AppServices {
    pub calendar: CalendarRepository,
    pub rule: RuleRepository,
}

impl AppServices {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            // poolは内部で参照カウントされているのでcloneしても低コスト
            calendar: CalendarRepository::new(pool.clone()),
            rule: RuleRepository::new(pool),
        }
    }
}

// =====================
// greet
// =====================
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// カレンダーデータを保存するコマンド
/// トランザクションを使用して整合性を保ちます

// =====================
// Tauri エントリポイント
// =====================
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            tauri::async_runtime::block_on(async {
                // --- app_data_dir を取得 ---
                let app_data_dir = app
                    .path()
                    .app_data_dir()
                    .expect("failed to get app data dir");

                // --- ディレクトリ作成（冪等） ---
                fs::create_dir_all(&app_data_dir)
                    .expect("failed to create app data dir");

                // --- DB パス生成 ---
                let db_path = app_data_dir.join("app.db");

                println!("Using DB at: {}", db_path.display());

                // --- DB 接続設定 ---
                let options = SqliteConnectOptions::new()
                    .filename(&db_path)
                    .create_if_missing(true); // <--- これが重要！ファイルがなければ作る

                // --- DB 接続 ---
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect_with(options) // connect ではなく connect_with を使う
                    .await
                    .expect("failed to open db");

                // 2. テーブル
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .expect("failed to run migrations");

                let services = AppServices::new(pool);

                // --- State に登録 ---
                app.manage(services);
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            application::commands::create_new_plan,
            application::commands::list_all_plans,
            application::commands::delete_plan,
            application::commands::get_plan_config,
            application::commands::add_staff_group,
            application::commands::delete_staff_group,
            application::commands::update_group_name,
            application::commands::add_staff_member,
            application::commands::delete_staff_member,
            application::commands::update_member_name,
            application::commands::add_weekly_rule,
            application::commands::delete_weekly_rule,
            application::commands::update_rule_name,
            application::commands::add_rule_assignment,
            application::commands::delete_assignment,
            application::commands::get_calendar_state,
            application::commands::derive_monthly_shift,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

