use serde::Serialize;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, FromRow};
use tauri::State;
use std::fs;

use tauri::Manager;

// =====================
// greet（既存）
// =====================
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// =====================
// DB用構造体
// =====================
#[derive(Serialize, FromRow)]
struct Item {
    id: i64,
    value: String,
}

// =====================
// DB操作 command
// =====================
#[tauri::command]
async fn add_item(
    value: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    sqlx::query("INSERT INTO items (value) VALUES (?1)")
        .bind(value)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn list_items(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Item>, String> {
    let items = sqlx::query_as::<_, Item>(
        "SELECT id, value FROM items ORDER BY id DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(items)
}

#[tauri::command]
async fn delete_item(
    id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM items WHERE id = ?1")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// =====================
// Tauri エントリポイント
// =====================
/*
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        let database_url = format!("sqlite:{}", db_path.display());

        println!("Using DB at: {}", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("failed to open db");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                value TEXT NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .expect("failed to create table");

        tauri::Builder::default()
            .manage(pool)
            .plugin(tauri_plugin_opener::init())
            .invoke_handler(tauri::generate_handler![
                greet,
                add_item,
                list_items,
                delete_item
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
*/


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
                let database_url = format!("sqlite:{}", db_path.display());

                println!("Using DB at: {}", db_path.display());

                // --- DB 接続 ---
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&database_url)
                    .await
                    .expect("failed to open db");

                // --- 初期化（冪等） ---
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS items (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        value TEXT NOT NULL
                    )
                    "#
                )
                .execute(&pool)
                .await
                .expect("failed to create table");

                // --- State に登録 ---
                app.manage(pool);
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            add_item,
            list_items,
            delete_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

