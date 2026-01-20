-- Add migration script here
-- 1. プラン（設定のまとまり）
-- 複数のルールセットを管理したいという要望に対応するテーブル
CREATE TABLE plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,           -- 例: "通常シフト", "繁忙期シフト"
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 2. スタッフグループ
-- AppState.staff_groups に相当
CREATE TABLE staff_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id INTEGER NOT NULL,     -- どのプランに属するか
    name TEXT NOT NULL,           -- 例: "キッチン", "ホール"
    sort_order INTEGER NOT NULL,  -- Vecの並び順を復元するために必要
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
);

-- 3. スタッフ（スロット）
-- StaffGroup.slots に相当
CREATE TABLE staff_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,    -- どのグループに属するか
    name TEXT NOT NULL,           -- 例: "田中", "佐藤"
    sort_order INTEGER NOT NULL,  -- グループ内での並び順 (Hollのindex対応用)
    FOREIGN KEY (group_id) REFERENCES staff_groups(id) ON DELETE CASCADE
);

-- 4. 週間ルール
-- AppState.rules に相当
CREATE TABLE weekly_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id INTEGER NOT NULL,
    name TEXT NOT NULL,           -- 例: "基本週", "隔週パターンA"
    sort_order INTEGER NOT NULL,
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
);

-- 5. ルールの詳細設定 (Holl)
-- WeekSchedule 内の DayShiftIds に相当
-- 正規化して保存します
CREATE TABLE rule_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    weekly_rule_id INTEGER NOT NULL,

    -- 曜日 (0:Mon, 1:Tue ... 6:Sun)
    weekday INTEGER NOT NULL,

    -- 時間帯 (0:Morning, 1:Afternoon)
    shift_time_type INTEGER NOT NULL,

    -- Holl の中身
    -- ここで重要なのは、staff_group_id は DBのID を指すが、
    -- shift_staff_index は「並び順(sort_order)」を指すという点です。
    target_group_id INTEGER NOT NULL,
    target_member_index INTEGER NOT NULL,

    FOREIGN KEY (weekly_rule_id) REFERENCES weekly_rules(id) ON DELETE CASCADE,
    FOREIGN KEY (target_group_id) REFERENCES staff_groups(id)
);

