// --- カレンダー管理層の実装 ---
use shift_calendar::{
    self,
    shift_gen::{
        DayRule,
        Incomplete, 
        ShiftHoll, 
        StaffGroupList,  
        WeekRule, 
        WeekRuleTable,
        WeekDecidedShift,
        gen_one_week_shift,
    }
};

#[derive(Debug)]
pub enum WeekStatus {
    Active { 
        logical_delta: LogicalDelta,
        // week_decided: WeekDecidedShift<'a>
    },
    Skipped,
}

// 1970年以前のシフトには対応しません（必要ない故）
// 絶対週番号を型エイリアスとして定義（わかりやすくするため）
pub type AbsWeek = usize;
pub type LogicalDelta = usize;

/// 確定した予定を入れます
pub struct ShiftCalendarManager {
    base_abs_week: AbsWeek,
    initial_delta: LogicalDelta,

    timeline: Vec<WeekStatus>, // 実週番号 -> 状態
}

#[derive(Debug)]
pub enum AppendWeekErrorKind {
    /// 予定の上書きエラー
    AttemptedToOverwrite,
    /// 連続しない予定エラー
    NotConsecutiveShifts,
    /// 加減突破
    UnderFlow,
}

impl ShiftCalendarManager {

    pub fn new(base_abs_week: AbsWeek, initial_delta: LogicalDelta) -> Self {
        // コンストラクターがエラーを判定
        Self { base_abs_week, initial_delta, timeline: Vec::new() }
    }

    fn delta_to_abs(&self, delta: LogicalDelta) -> AbsWeek {
        let abs_week = delta + (self.base_abs_week - self.initial_delta);

        abs_week
    }

    fn abs_to_delta(&self, abs_week: AbsWeek) -> Result<LogicalDelta, AppendWeekErrorKind> {
        if abs_week < (self.base_abs_week - self.initial_delta) {
            Err(AppendWeekErrorKind::UnderFlow)
        } else {
            Ok(abs_week - (self.base_abs_week - self.initial_delta))
        }
    }

    fn delta_to_index(&self, delta: LogicalDelta) -> Result<usize, AppendWeekErrorKind> {
        if delta < self.initial_delta {
            Err(AppendWeekErrorKind::UnderFlow) // out of index
        }else{
            Ok(delta - self.initial_delta)
        }
    }

    fn abs_to_index(&self, abs_week: AbsWeek) -> Result<usize, AppendWeekErrorKind> {
        self.delta_to_index(self.abs_to_delta(abs_week)?)
    }

    /// すでに作成したリストのなかにtarget_abs_weekを含み
    /// かつそれが変更される場合はエラーを返却する.
    ///
    /// [A | S]
    ///
    /// Ok
    /// ```
    ///  0  1  2  3  4  5
    /// [A, S, A, A]       self
    ///     I  I  I    
    ///    [S, A, A, N, N] shift you want to append
    /// target_abs_week: 1
    /// ```
    ///
    /// Err(AttemptedToOverwrite)
    /// ```
    ///  0  1  2  3  4  5
    /// [A, S, A, A] E  E  self
    ///     I  I  N      
    ///    [S, A, S, N, N] shift you want to append
    /// target_abs_week: 1
    /// ```
    ///
    /// if self.timeline[delta_to_abs(initial_delta)..], [0..self.timeline.len() - delta_to_abs(initial_delta)]
    ///
    /// Ok
    /// ```
    ///  0  1  2  3  4
    /// [A, S, A, A] E              :self
    ///              I
    ///             [S, A, S, N, N] :shift you want to append
    /// target_abs_week: 4
    /// ```
    ///
    /// Err(NotConsecutiveShifts)
    /// ```
    ///  0  1  2  3  4  5
    /// [A, S, A, A] E  E              :self
    ///              E [S, A, S, N, N] :shift you want to append
    /// target_abs_week: 5
    /// ```
    /// if self.timeline.len() < target_abs_week -> Error
    pub fn append_check(
        &mut self,
        target_abs_week: AbsWeek,
        skip_flags: &[bool]
    ) -> Result<(), AppendWeekErrorKind> {
        if self.timeline.len() < self.abs_to_delta(target_abs_week)? {
            return Err(AppendWeekErrorKind::NotConsecutiveShifts);
        }

        if self
            .timeline[self.abs_to_index(target_abs_week)?..]
            .iter()
            .zip(
                skip_flags
            ).all(|(week_status, is_skipped)|
                matches!(week_status, WeekStatus::Skipped) == *is_skipped
            )
        {
            // Ok
        } else {
            return Err(AppendWeekErrorKind::AttemptedToOverwrite);
        }
        Ok(())
    }

    pub fn append_week(
        &mut self,
        is_skipped: bool,
    ) {
        // 1. 次のdeltaを計算
        let next_delta = match self.timeline.last() {
            Some(WeekStatus::Active { logical_delta}) => logical_delta + 1,
            Some(WeekStatus::Skipped) => {
                self.find_last_active_delta().map(|d| d + 1).unwrap_or(self.initial_delta)
            },
            None => self.initial_delta, // 初回
        };

        // 2. スロット作成
        let slot = if is_skipped {
            WeekStatus::Skipped
        } else {
            WeekStatus::Active { 
                logical_delta: next_delta,
            }
        };

        self.timeline.push(slot);
    }

    /// シフトの導出
    /// base_abs_weekを下回る場合でも返せる場合があるが
    /// skipを使うことで週の途中からルールを開始することは可能なので対応しない
    pub fn derive_shift<'a>(
        &self,
        week_rule_table: & WeekRuleTable<'a, Incomplete>,
        staff_group_list: &'a StaffGroupList,
        gen_week_abs: AbsWeek,  // 生成の始点となる絶対週
        gen_range: usize,       // 何週間分のシフトを作成するか
    ) -> Vec<Option<WeekDecidedShift<'a>>>{
        if let Ok(index) = self.abs_to_index(gen_week_abs) {
            if index + gen_range < self.timeline.len() {
                self.timeline[
                    index..index + gen_range
                ].iter().map(|i|{
                    if let WeekStatus::Active { logical_delta } = i {
                        Some(gen_one_week_shift(week_rule_table, staff_group_list, *logical_delta))
                    } else {
                        None
                    }
                }).collect()
            } else if index < self.timeline.len() {
                self.timeline[
                    index..
                ].iter().map(|i|{
                    if let WeekStatus::Active { logical_delta } = i {
                        Some(gen_one_week_shift(week_rule_table, staff_group_list, *logical_delta))
                    } else {
                        None
                    }
                }).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// 直近の有効なDeltaを探すヘルパー
    fn find_last_active_delta(&self) -> Option<LogicalDelta> {
        self.timeline.iter().rev().find_map(|slot| match slot {
            WeekStatus::Active {
                logical_delta, } => Some(*logical_delta),
            WeekStatus::Skipped => None,
        })
    }

    /// 【重要】指定した絶対週以降をすべて削除する（Truncate）
    /// 配列を短くするだけなので極めて高速かつ安全
    pub fn truncate_from(&mut self, target_abs_week: AbsWeek) {
        if target_abs_week < self.base_abs_week {
            // 開始地点より前を指定されたら全消し
            self.timeline.clear();
            // 必要なら start_abs_week 自体を書き換えるロジックも検討
            return;
        }

        let keep_len = (target_abs_week - self.base_abs_week) as usize;
        if keep_len < self.timeline.len() {
            self.timeline.truncate(keep_len);
        }
    }
}


#[cfg(test)]
mod shift_calendar_manager {
    use crate::shift_calendar_manager::{AppendWeekErrorKind, ShiftCalendarManager};

    /// 正しくシフトカレンダーに設定できるか？
    #[test] 
    fn test00() {
        let mut shift_calendar_manager 
            = ShiftCalendarManager::new(2000, 0);

        let mut r = shift_calendar_manager
            .append_check(2000, &[
                false, // 2000
                false, // 2001
                true,  // 2002 skip!
                false  // 2003
            ]);

        // 何もないリストへ追加可能をチェック
        // ここではOkと変えるはず
        assert!(matches!(r, Ok(())));

        shift_calendar_manager.append_week(false);
        shift_calendar_manager.append_week(false);
        shift_calendar_manager.append_week(true);
        shift_calendar_manager.append_week(false);  // 2003

        let r = shift_calendar_manager
            .append_check(2001, &[
                false, // 2001
                true,  // 2002
                false, // 2003
                false  // 2004 (new)
            ]);

        shift_calendar_manager.append_week(false);  // 2004

        assert!(matches!(r, Ok(())));

        let r = shift_calendar_manager
            .append_check(2005, &[
                false, // 2005
            ]);

        shift_calendar_manager.append_week(false);  // 2005
        assert!(matches!(r, Ok(())));

        let r = shift_calendar_manager
            .append_check(2007, &[
                false, // 2005
            ]);

        assert!(matches!(r, Err(AppendWeekErrorKind::NotConsecutiveShifts)));

        let r = shift_calendar_manager
            .append_check(2001, &[
                true, // 2001
                true,  // 2002
                false, // 2003
                false  // 2004 (new)
            ]);

        assert!(matches!(r, Err(AppendWeekErrorKind::AttemptedToOverwrite)));
    }

    use shift_calendar::{
        self,
        shift_gen::{
            DayRule,
            Incomplete, 
            ShiftHoll, 
            StaffGroup,
            StaffGroupList,  
            WeekRule, 
            WeekRuleTable,
            WeekDecidedShift,
        }
    };

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Group {
        staff: Vec<String>,
    }

    use std::{collections::BTreeMap, fmt::Debug};
    type Config = BTreeMap<String, Group>;

    /// char to shiftholl 
    fn c2h<'a>(type_char:char, id:usize) -> Option<ShiftHoll<'a, Incomplete>> {
        match type_char {
            'a' => Some(ShiftHoll::new(0, id)),
            'b' => Some(ShiftHoll::new(1, id)),
            'c' => Some(ShiftHoll::new(2, id)), // for incorrect test case
            _ => None
        }
    }

    // test macro
    macro_rules! h {
        ($id:ident) => {{
            let s = stringify!($id);
            let mut chars = s.chars();
            let c = chars.next().expect("empty ident");
            let n: usize = chars.as_str().parse().expect("invalid number");
            c2h(c, n).unwrap()
        }};
    }

    macro_rules! day_rule {
        (
            m[$($m:ident),* $(,)?],
            a[$($a:ident),* $(,)?]
        ) => {
            DayRule {
                shift_morning: vec![$(h!($m)),*],
                shift_afternoon: vec![$(h!($a)),*],
            }
        };
    }

    macro_rules! week_rule {
        (
            $(
                $day:ident :
                m[$($m:ident),* $(,)?],
                a[$($a:ident),* $(,)?]
            ),* $(,)?
        ) => {
            WeekRule([
                $(
                    day_rule!(m[$($m),*], a[$($a),*])
                ),*
            ])
        };
    }

    // impl<'a> Debug for WeekDecidedShift<'a> {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         
    //     }
    // }

    #[test]
    fn test01() {

        let mut shift_calendar_manager 
            = ShiftCalendarManager::new(2000, 0);

        let week_rule0 = week_rule![
            mon: m[a0, b0],  a[b1],
            tue: m[],        a[a1],
            wed: m[],        a[],
            thu: m[b4],      a[],
            fri: m[b5, b2],  a[a3, b3, a2],
            sat: m[],        a[],
            sun: m[],        a[],
        ];
        let week_rule1 = week_rule![
            mon: m[a2, b3],  a[b2],
            tue: m[],        a[b4],
            wed: m[],        a[],
            thu: m[a1],      a[],
            fri: m[b1, b3],  a[b5, a0, b0],
            sat: m[],        a[],
            sun: m[],        a[],
        ];

        let week_rule_table = WeekRuleTable(vec![week_rule0, week_rule1]);

        // Read Staff info from test.toml file
        let s = std::fs::read_to_string("test.toml").unwrap();
        let groups: Config = toml::from_str(&s).unwrap();
        let mut staff_group_a = StaffGroup::new("group a");

        for name in &groups["A"].staff {
            staff_group_a.add_staff(name);
        }
        let mut staff_group_b = StaffGroup::new("group b");
        for name in &groups["B"].staff {
            staff_group_b.add_staff(name);
        }

        let mut staff_group_list = StaffGroupList::new();

        staff_group_list.add_staff_group(staff_group_a);
        staff_group_list.add_staff_group(staff_group_b);

        let r = shift_calendar_manager
            .append_check(2000, &[
                false, // 2001
                false,  // 2002
                true, // 2003
                false  // 2004 (new)
            ]);

        // then 
        shift_calendar_manager.append_week(false);
        shift_calendar_manager.append_week(false);
        shift_calendar_manager.append_week(true);
        shift_calendar_manager.append_week(false); // ここで設定されたら決定

        assert!(matches!(r, Ok(())));

        let gen_week_abs = 2000; // 生成をしたい週の最初
        let gen_range = 5;       // 何週間分
        // 内部的にはシフトは決定済み
        let a = shift_calendar_manager
            .derive_shift(
                &week_rule_table,
                &staff_group_list, 
                gen_week_abs, 
                gen_range
            );

        for i in a {
            println!("Week Shift");
            if let Some(week) = i {
                println!("{:?}", week);
            } else {
                println!("----------------------------");
            }
        }
        println!("finish!");
    }
}
