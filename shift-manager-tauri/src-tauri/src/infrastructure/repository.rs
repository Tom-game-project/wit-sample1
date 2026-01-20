use sqlx::{ 
    SqlitePool,
    sqlite::SqlitePoolOptions,
    FromRow
};

use crate::domain::models::{ShiftCalendarManager, WeekStatus};

pub struct CalendarRepository {
    pool: SqlitePool,
}

// =====================
// DB読み込み用ヘルパー構造体
// =====================

// 親テーブル読み込み用
#[derive(FromRow)]
struct CalendarHeaderRow {
    id: i64,
    base_abs_week: i64,
    initial_delta: i64,
}

// 子テーブル読み込み用
#[derive(FromRow)]
struct WeekStatusRow {
    #[allow(dead_code)] // クエリで取得するがRust側で使わない場合用
    week_offset: i64,
    status_type: String,
    logical_delta: Option<i64>,
}

impl CalendarRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, manager: &ShiftCalendarManager) -> Result<i64, String> {
        // 1. トランザクション開始
        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        // 2. 親テーブル (shift_calendars) へ保存
        // usize は i64 として保存します
        let row_id = sqlx::query(
            "INSERT INTO shift_calendars (
                base_abs_week,
                initial_delta
            ) VALUES (?1, ?2)",
        )
        .bind(manager.base_abs_week as i64)
        .bind(manager.initial_delta as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();

        // 3. 子テーブル (weekly_statuses) へ保存
        for (index, status) in manager.timeline.iter().enumerate() {
            let (status_type, logical_delta) = match status {
                WeekStatus::Active { logical_delta } => ("Active", Some(*logical_delta as i64)),
                WeekStatus::Skipped => ("Skipped", None),
            };

            sqlx::query(
                "INSERT INTO weekly_statuses (
                    calendar_id,
                    week_offset,
                    status_type,
                    logical_delta
                ) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(row_id)
            .bind(index as i64)
            .bind(status_type)
            .bind(logical_delta)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        // 4. トランザクションコミット
        tx.commit().await.map_err(|e| e.to_string())?;

        Ok(row_id)

    }

    pub async fn find_latest(&self) -> Result<Option<ShiftCalendarManager>, String> {
        // 1. 最新の親データを取得
        let header_opt: Option<CalendarHeaderRow> = sqlx::query_as::<sqlx::Sqlite, CalendarHeaderRow>(
            "SELECT id, base_abs_week, initial_delta FROM shift_calendars ORDER BY id DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        let header = match header_opt {
            Some(h) => h,
            None => return Ok(None), // データがない場合
        };

        // 2. 関連する子データを取得（week_offset順）
        let rows: Vec<WeekStatusRow> = sqlx::query_as::<sqlx::Sqlite, WeekStatusRow>(
            "SELECT week_offset, status_type, logical_delta FROM weekly_statuses WHERE calendar_id = ?1 ORDER BY week_offset ASC"
        )
        .bind(header.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        // 3. Rustの構造体に再構築
        let timeline: Vec<WeekStatus> = rows
            .into_iter()
            .map(|row| match row.status_type.as_str() {
                "Active" => WeekStatus::Active {
                    logical_delta: row.logical_delta.unwrap_or(0) as usize,
                },
                "Skipped" => WeekStatus::Skipped,
                _ => WeekStatus::Skipped, // 未知の値へのフォールバック（本来はエラー処理推奨）
            })
            .collect();

        Ok(Some(ShiftCalendarManager {
            id: Some(header.id),
            base_abs_week: header.base_abs_week as usize,
            initial_delta: header.initial_delta as usize,
            timeline,
        }))
    }
}

#[cfg(test)]
mod repository_tests {
    use super::*;
    use crate::domain::models::{ShiftCalendarManager, WeekStatus};
    use sqlx::sqlite::SqlitePoolOptions;

    // テスト用のDBセットアップ（テーブル作成）
    async fn setup_test_db() -> SqlitePool {
        // メモリ上のDBを使用（テストが終わると消える）
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create memory pool");

        // テーブル作成（本番と同じSQLを実行）

        // 2. 子テーブル
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_save_and_find_latest() {
        // 1. 準備 (Arrange)
        let pool = setup_test_db().await;
        let repository = CalendarRepository::new(pool);

        // テストデータ作成
        let input_data = ShiftCalendarManager {
            id: None, // 保存前はNone
            base_abs_week: 100,
            initial_delta: 5,
            timeline: vec![
                WeekStatus::Active { logical_delta: 1 },
                WeekStatus::Skipped,
                WeekStatus::Active { logical_delta: 2 },
            ],
        };

        // 2. 実行 (Act)
        // 保存
        let saved_id = repository.save(&input_data).await.expect("Failed to save");
        
        // 最新取得
        let fetched_opt = repository.find_latest().await.expect("Failed to find latest");

        // 3. 検証 (Assert)
        assert!(fetched_opt.is_some());
        let fetched_data = fetched_opt.unwrap();

        // IDが入っているか確認
        assert_eq!(fetched_data.id, Some(saved_id));

        // データの中身が一致するか確認
        assert_eq!(fetched_data.base_abs_week, input_data.base_abs_week);
        assert_eq!(fetched_data.initial_delta, input_data.initial_delta);
        assert_eq!(fetched_data.timeline, input_data.timeline);
    }
}
