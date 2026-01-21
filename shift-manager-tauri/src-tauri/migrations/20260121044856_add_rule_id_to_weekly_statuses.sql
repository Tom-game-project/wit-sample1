-- Add migration script here

DROP TABLE IF EXISTS weekly_statuses;
DROP TABLE IF EXISTS shift_calendars;

CREATE TABLE IF NOT EXISTS shift_calendars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    plan_id INTEGER NOT NULL,

    base_abs_week INTEGER NOT NULL,
    initial_delta INTEGER NOT NULL,
    -- プランが消えたらカレンダーも消えるように設定
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
);

-- 2. 週ごとの状態 (Rule IDを追加)
CREATE TABLE weekly_statuses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    calendar_id INTEGER NOT NULL,
    week_offset INTEGER NOT NULL,
    status_type TEXT NOT NULL CHECK (status_type IN ('Active', 'Skipped')),

    -- Activeのときのみ値が入る
    logical_delta INTEGER,

    -- ★追加: 適用するルールのID (Activeのときのみ)
    rule_id INTEGER,

    FOREIGN KEY (calendar_id) REFERENCES shift_calendars(id) ON DELETE CASCADE,
    -- TODO:ルールが消えたらSet Nullにするか、Cascadeするかは要件次第ですが、
    -- ここでは整合性重視で外部キー制約をつけておきます
    FOREIGN KEY (rule_id) REFERENCES weekly_rules(id)
);
