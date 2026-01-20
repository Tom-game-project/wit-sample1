-- Add migration script here

-- shift_calendars テーブル
CREATE TABLE IF NOT EXISTS shift_calendars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    base_abs_week INTEGER NOT NULL,
    initial_delta INTEGER NOT NULL
);

-- weekly_statuses テーブル
CREATE TABLE IF NOT EXISTS weekly_statuses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    calendar_id INTEGER NOT NULL,
    week_offset INTEGER NOT NULL,
    status_type TEXT NOT NULL CHECK (status_type IN ('Active', 'Skipped')),
    logical_delta INTEGER,
    FOREIGN KEY (calendar_id) REFERENCES shift_calendars(id)
);
