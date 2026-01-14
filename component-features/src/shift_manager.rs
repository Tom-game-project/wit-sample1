wit_bindgen::generate!({
    world: "shift-manager-world",
    generate_all
});

struct Component;

use exports::component::component_features::shift_manager::{ 
    ShiftWeekday, 
    ShiftTime,
    GuestShiftManager,
    Guest,
    // ==== staff group data ====
    StaffGroup,
    StaffInfo,
    // ==== weekly rule data ====
    WeeklyRule,
    WeekSchedule,
    DayShiftIds,
    Holl,
    // ==== out ====
    WeeklyShiftOut
};

use shift_calendar::{
    self,
    shift_gen::{
        DayDecidedShift, DayRule, Incomplete, ShiftHoll, Staff, WeekDecidedShift, WeekRule 
    }
};

use std::{
    cell::RefCell,
};

use crate::{
    shift_calendar_manager::{
        AbsWeek, 
        ShiftCalendarManager
    },
    shift_manager::exports::component::component_features::shift_manager::{
        DailyShiftOut, 
        StaffPillOut
    }
};

use crate::shift_gen::dummy::logger::logger::log;

use chrono::{NaiveDate, Duration, Datelike};

// --------------------------------------------------------
// 1. Staff Groups Definition
// --------------------------------------------------------
//
// スタッフ名を格納するスロット
// struct StaffGroup {
//     name: String,         // 例: "Kitchen"
//     slots: Vec<StaffInfo>,   // 例: ["Leader", "Sub", ""]
// }
// 
// struct StaffInfo(String);

impl StaffGroup {
    fn add_slot(&mut self) {
        self.slots.push(StaffInfo{name: String::from("")});
    }

    fn remove_slot(&mut self, slot_idx: u32) {
        self.slots.remove(slot_idx as usize);
    }

    fn update_memo(&mut self, staff_slot_index: u32, name:String) {
        if let Some(a) = self.slots.get_mut(staff_slot_index as usize) {
            a.name = name;
        }
    }
}

// --------------------------------------------------------
// 2. Weekly Rules Definition
// --------------------------------------------------------
// "a0", "b1" などのIDは String として扱います
//
// 週ごとのシフトホール格納スロット
// struct WeeklyRule {
//     name: String,         // 例: "Standard Week"
//     schedule: WeekSchedule
// }

// struct WeekSchedule {
//     mon: DayShiftIds,
//     tue: DayShiftIds,
//     wed: DayShiftIds,
//     thu: DayShiftIds,
//     fri: DayShiftIds,
//     sat: DayShiftIds,
//     sun: DayShiftIds,
// }

impl WeekSchedule {
    fn new() -> Self {
        Self {
                mon: DayShiftIds::new(), 
                tue: DayShiftIds::new(), 
                wed: DayShiftIds::new(), 
                thu: DayShiftIds::new(), 
                fri: DayShiftIds::new(), 
                sat: DayShiftIds::new(), 
                sun: DayShiftIds::new(), 
        }
    }

    // WeeklyRuleに新しいhollを追加
    fn add_week_rule_assignment (
        &mut self,
        day: ShiftWeekday,
        shift_time: ShiftTime,
        new_holl: Holl
    ) {
        let day = day.extract_mut_day_shift_ids(self);

        match shift_time {
            ShiftTime::Morning => {
                day.m.push(new_holl);
            }
            ShiftTime::Afternoon => {
                day.a.push(new_holl);
            } 
        }
    }

    fn remove_week_rule_assignment(
        &mut self, 
        day: ShiftWeekday,
        shift_time: ShiftTime,
        index: u32
    ) {
        let day = day.extract_mut_day_shift_ids(self);

        match shift_time {
            ShiftTime::Morning => {
                day.m.remove(index as usize);
            }
            ShiftTime::Afternoon => {
                day.a.remove(index as usize);
            } 
        }
    }

    fn get_week_rule_assignment (
        &self,
        day: ShiftWeekday,
        shift_time: ShiftTime,
    ) -> Vec<Holl> {
        let day = day.extract_day_shift_ids(self);

        match shift_time {
            ShiftTime::Morning => {
                day.m.clone()
            }
            ShiftTime::Afternoon => {
                day.a.clone()
            } 
        }
    }
}

impl ShiftWeekday {
    fn extract_day_shift_ids<'a>(&self, week_schedule: &'a WeekSchedule) -> &'a DayShiftIds {
        match self {
            ShiftWeekday::Mon => &week_schedule.mon,
            ShiftWeekday::Tue => &week_schedule.tue,
            ShiftWeekday::Wed => &week_schedule.wed,
            ShiftWeekday::Thu => &week_schedule.thu,
            ShiftWeekday::Fri => &week_schedule.fri,
            ShiftWeekday::Sat => &week_schedule.sat,
            ShiftWeekday::Sun => &week_schedule.sun,
        }
    }

    fn extract_mut_day_shift_ids<'a>(&self, week_schedule: &'a mut WeekSchedule) -> &'a mut DayShiftIds{
        match self {
            ShiftWeekday::Mon => &mut week_schedule.mon,
            ShiftWeekday::Tue => &mut week_schedule.tue,
            ShiftWeekday::Wed => &mut week_schedule.wed,
            ShiftWeekday::Thu => &mut week_schedule.thu,
            ShiftWeekday::Fri => &mut week_schedule.fri,
            ShiftWeekday::Sat => &mut week_schedule.sat,
            ShiftWeekday::Sun => &mut week_schedule.sun,
        }
    }
}

impl WeeklyRule {
    fn new() -> Self{
        Self { 
            name: String::from(""),
            schedule: WeekSchedule::new()
        }
    }

    fn change_name(&mut self, name: String) {
        self.name = name;
    }
}

// 日ごとのシフトホール格納スロット
// struct DayShiftIds {
//     m: Vec<Holl>,       // 午前シフトのIDリスト ["a0", "b1"]
//     a: Vec<Holl>,       // 午後シフトのIDリスト []
// }

impl DayShiftIds {
    fn new() -> Self {
        Self { m: vec![], a: vec![] }
    }

    /// shift_calendarが処理できる型に変換する
    fn day_shift_ids_into_day_rule<'a>(&self) -> DayRule<'a, Incomplete> {
        DayRule {
            shift_morning: self.m.iter().map(|h| h.into_shift_holl()).collect(),
            shift_afternoon: self.a.iter().map(|h| h.into_shift_holl()).collect(),
        }
    }
}

// シフトホール
// struct Holl {
//     staff_group_id:u32, // スタッフグループを指す
//     shift_staff_index:u32, // シフトのルールを司るindex
// }

impl Holl {
    fn into_shift_holl<'a>(&self) -> ShiftHoll<'a, Incomplete> {
        ShiftHoll::new(
            self.staff_group_id as usize,
            self.shift_staff_index as usize
        )
    }
}

// --------------------------------------------------------
// 4. Root State (Entire Application State)
// --------------------------------------------------------
//
// この構造体がアプリ全体の状態を管理する
// このモジュールはwit/depsに依存する
// この構造体はstatic life time
struct AppState {
    staff_groups: RefCell<Vec<StaffGroup>>,
    rules: RefCell<Vec<WeeklyRule>>,

    year: RefCell<u32>,            // 例: 2026
    month: RefCell<u32>,           // 例: 0 (Jan) - 11 (Dec)

    // Key: "YYYY-MM-DD"
    // 実際に生成されたカレンダー
    schedule_data: RefCell<ShiftCalendarManager>,
}

impl GuestShiftManager for AppState {
    fn new() -> Self {
        Self {
            staff_groups: RefCell::new(vec![]),
            rules: RefCell::new(vec![]),
            year: RefCell::new(2026), // TODO 初期値はdeps内の関数から取得する必要あり
            month: RefCell::new(1),   // TODO  初期値はdeps内の関数から取得する必要あり
            schedule_data: RefCell::new(
                ShiftCalendarManager::new(
                    // TODO
                    // TODO
                    2926, //base_abs_week,
                    0 // initial_delta
                )
            )
        }
    }

    fn add_new_group(&self) {
        let staff_group_length = self.staff_groups.borrow().len();
        self.staff_groups.borrow_mut().push(
            StaffGroup { 
                name: format!("Group{}", staff_group_length),
                slots: vec![]
            }
        );
    }

    fn remove_group(&self, index: u32) {
        self.staff_groups.borrow_mut().remove(index as usize);
    }

    fn update_group_name(&self, index: u32, name: String) {
        if let Some(a) = self
            .staff_groups
            .borrow_mut()
            .get_mut(index as usize) {
                a.name = name;
        }
    }

    fn add_slot(&self, group_idx: u32) {
        if let Some(a) = self
            .staff_groups
            .borrow_mut()
            .get_mut(group_idx as usize)
        {
            a.add_slot();
        }
    }

    fn remove_slot(&self,group_idx:u32,slot_idx:u32,){
        if let Some(a) =self
            .staff_groups
            .borrow_mut()
            .get_mut(group_idx as usize) {
                a.remove_slot(slot_idx);
        }
    }

    fn update_slot_memo(&self, group_idx:u32, slot_idx:u32, memo: String) {
        if let Some(a) = self
            .staff_groups
            .borrow_mut()
            .get_mut(group_idx as usize)
        {
            a.update_memo(slot_idx, memo);
        }
    }

    fn add_week(&self) {
        self
            .rules
            .borrow_mut()
            .push(WeeklyRule::new());
    }

    fn remove_rule(&self, index: u32) {
        self.rules.borrow_mut().remove(index as usize);
    }

    fn update_rule_name(&self, index: u32, name: String) {
        if let Some(a) =self
            .rules
            .borrow_mut()
            .get_mut(index as usize) {
            a.change_name(name);
        }
    }

    fn add_rule_assignment(
        &self,
        rule_idx:u32,
        day: ShiftWeekday,
        shift_time: ShiftTime,
        staff_group_id:u32,
        shift_staff_index:u32,
    )
    {
        if let Some(weekly_rule) = self
            .rules
            .borrow_mut()
            .get_mut(
            rule_idx as usize
        ){ 
            weekly_rule
                .schedule
                .add_week_rule_assignment(
                    day,
                    shift_time, 
                    Holl { staff_group_id, shift_staff_index }
                );
        }
    }

    fn remove_rule_assignment(
        &self,
        rule_idx: u32, 
        day: ShiftWeekday, 
        shift_time: ShiftTime, 
        index: u32) {
        if let Some(weekly_rule) = self
            .rules
            .borrow_mut()
            .get_mut(
            rule_idx as usize
        ){ 
            weekly_rule
                .schedule
                .remove_week_rule_assignment(
                    day, 
                    shift_time,
                    index
                );
        }
    }

    fn get_rule_assignment(
        &self,
        rule_idx: u32,
        day: ShiftWeekday,
        shift_time: ShiftTime,) -> Option<Vec<Holl>> {
        if let Some(weekly_rule) = self
            .rules
            .borrow_mut()
            .get_mut(
            rule_idx as usize
        ){ 
            Some(
                weekly_rule
                    .schedule
                    .get_week_rule_assignment(
                        day, 
                        shift_time,)
            )
        } else {
            None
        }
    }

    fn change_prev_month(
        &self) {
        let mut month = self.month.borrow_mut();
        let mut year = self.year.borrow_mut();

        if *month == 0 {
            *month = 11;
            *year -= 1;
        } else {
            *month -= 1;
        }
    }

    fn change_next_month(
        &self
    ) {
        let mut month = self.month.borrow_mut();
        let mut year = self.year.borrow_mut();

        if *month == 11 {
            *month = 0;
            *year += 1;
        } else {
            *month += 1;
        }
    }

    fn get_rules(&self) -> Vec<WeeklyRule> {
        self.rules.borrow().clone()
    }

    fn get_staff_groups(&self) -> Vec<StaffGroup> {
        self.staff_groups.borrow().clone()
    }

    fn get_year(&self) -> u32 {
        *self.year.borrow()
    }

    fn get_month(&self) -> u32 {
        *self.month.borrow()
    }

    fn get_monthly_shift(&self) -> Vec<Option<WeeklyShiftOut>> {
        let mut week_rule_table = 
            shift_calendar::shift_gen::WeekRuleTable::new();
        let mut staff_group_list = 
            shift_calendar::shift_gen::StaffGroupList::new();
        for i in self.get_rules() {
            week_rule_table.add_week_rule(WeekRule([
                i.schedule.mon.day_shift_ids_into_day_rule(),
                i.schedule.tue.day_shift_ids_into_day_rule(),
                i.schedule.wed.day_shift_ids_into_day_rule(),
                i.schedule.thu.day_shift_ids_into_day_rule(),
                i.schedule.fri.day_shift_ids_into_day_rule(),
                i.schedule.sat.day_shift_ids_into_day_rule(),
                i.schedule.sun.day_shift_ids_into_day_rule(),
            ]));
        }

        for i in self.get_staff_groups() {
            let mut staff_group = 
                shift_calendar::shift_gen::StaffGroup::new(&i.name);
            for j in &i.slots {
                staff_group.add_staff(&j.name);
            }
            staff_group_list.add_staff_group(staff_group);
        }

        let gen_week_abs = if let Some (a) = calculate_weeks_delta_from_base(
            self.get_year() as i32,
            self.get_month(),
            1
        ) { 
            a
        } else {
            return Vec::new();
        };

        log("get_monthly_shift");
        log(&format!("カレンダー最上部は {}", gen_week_abs));
        log(&format!("この月は　{}週間続きます", calculate_weeks_in_month(
                    self.get_year() as i32,
                    self.get_month())));

        self.schedule_data
            .borrow()
            .derive_shift(
                &week_rule_table,
                &staff_group_list,
                gen_week_abs,
                calculate_weeks_in_month(
                    self.get_year() as i32,
                    self.get_month()) as usize
            )
            .iter()
            .map(|a| {
                a.as_ref().map(|b| 
                    week_decided_shift_into_weekly_shift_out(&b)
                )
            }
            )
            .collect()
    }

    fn apply_month_shift(&self, skip_flags: Vec<bool>) {
        if let Some (gen_week_abs) =
            calculate_weeks_delta_from_base(
            self.get_year() as i32,
            self.get_month(),
            1
        ) {
            // TODO: Err処理をするapi設計に変える
            if let Err(e) = self.schedule_data.borrow_mut().apply_weeks(
                gen_week_abs, 
                &skip_flags
            ) {
                log(&format!("error occured {:?}", e));
            };
        }
    }

    fn get_skip_flags(
        &self
    ) -> Vec<bool> {
        let gen_week_abs = if let Some (a) = calculate_weeks_delta_from_base(
            self.get_year() as i32,
            self.get_month(),
            1
        ) { 
            a
        } else {
            return Vec::new();
        };
        log(&format!("get skip list: skip list: {:?}", self.schedule_data.borrow().get_timeline()));

        let ret_data = self
            .schedule_data
            .borrow()
            .get_skip_list_by_abs(
                gen_week_abs,
                calculate_weeks_in_month(
                    self.get_year() as i32,
                    self.get_month()
                ) as usize
            );
        log(&format!("get skip list: ret_data: {:?} year {} month {}, gen week abs {}", ret_data, self.get_year(), self.get_month(), gen_week_abs));

        ret_data
    }

    fn reset_from_this_month(&self) {
        if let Some (a) = calculate_weeks_delta_from_base(
            self.get_year() as i32,
            self.get_month(),
            1
        ) {
            self
                .schedule_data
                .borrow_mut()
                .truncate_from(a);
        }
    }
}

fn staff_into_staff_pill_out (staff: &Staff) -> StaffPillOut {
    StaffPillOut { 
        name: staff.name.clone(),
        staff_group_id: staff.group_id as u32, 
        staff_index: staff.id as u32
    }
}

fn day_decided_shift_into_daily_shift_out (day_decided_shift: &DayDecidedShift) -> DailyShiftOut {
    DailyShiftOut {
        m: day_decided_shift.shift_morning.iter().map(|staff| staff_into_staff_pill_out(staff)).collect(), 
        a: day_decided_shift.shift_afternoon.iter().map(|staff| staff_into_staff_pill_out(staff)).collect()
    }
}

fn week_decided_shift_into_weekly_shift_out<'a>(week_decided_shift: &WeekDecidedShift<'a>) -> WeeklyShiftOut {
    WeeklyShiftOut { 
        mon:day_decided_shift_into_daily_shift_out(&week_decided_shift.0[0]),
        tue:day_decided_shift_into_daily_shift_out(&week_decided_shift.0[1]), 
        wed:day_decided_shift_into_daily_shift_out(&week_decided_shift.0[2]),
        thu:day_decided_shift_into_daily_shift_out(&week_decided_shift.0[3]), 
        fri:day_decided_shift_into_daily_shift_out(&week_decided_shift.0[4]), 
        sat:day_decided_shift_into_daily_shift_out(&week_decided_shift.0[5]), 
        sun:day_decided_shift_into_daily_shift_out(&week_decided_shift.0[6]) 
    }
}


/// ある日がbase_weekから数えて何になるかを調べる関数
/// month(0-11)
fn calculate_weeks_delta_from_base(year: i32, month: u32, day: u32) -> Option<AbsWeek> {
    //     January 1970
    //          unix base
    //          v
    // Mo Tu We Th Fr Sa Su
    //           1  2  3  4 < base week = 0
    //  5  6  7  8  9 10 11               1
    // 12 13 14 15 16 17 18               2
    // 19 20 21 22 23 24 25               :
    // 26 27 28 29 30 31
    //
    // 1969/12/29 as week base

    // (unix_base_week: number,week_delta:  number)  Mo Tu We Th Fr Sa Su
    // (unix_base_week: 0,week_delta:            0)           1  2  3  4
    // (unix_base_week: 1,week_delta:            1)  5  6  7  8  9 10 11
    // (unix_base_week: 2,week_delta:         skip) 12 13 14 15 16 17 18
    // (unix_base_week: 3,week_delta:            2) 19 20 21 22 23 24 25
    // (unix_base_week: 4,week_delta:            3) 26 27 28 29 30 31

    let date1 = NaiveDate::from_ymd_opt(1969, 12, 29)
        .unwrap() /* safe unwrap */;

    if let Some(date2)  = NaiveDate::from_ymd_opt(year, month + 1, day) {
        let diff: Duration = date2 - date1;
        let weeks = diff.num_weeks();

        if weeks < 0 {
            None
        } else {
            Some(weeks as usize) 
        }
        
    } else {
        None
    }
}

/// 指定された年・月が、カレンダー上で何週（何行）になるかを計算する
/// ※ month: 0 (1月) 〜 11 (12月)
/// ※ 月曜始まり (Monday start) 前提
pub fn calculate_weeks_in_month(year: i32, month: u32) -> u32 {
    // 1. その月の1日を取得
    // NaiveDate は 1-12 月を期待するため、引数 month(0-11) に +1 する
    let first_day = NaiveDate::from_ymd_opt(year, month + 1, 1)
        .expect("Invalid date provided (month should be 0-11)");

    // 2. その月の日数を計算
    // 翌月の1日を取得して差分を取る
    // month が 11 (12月) の場合は翌年、それ以外は同じ年の month + 2 月
    let next_month_date = if month == 11 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 2, 1).unwrap()
    };
    
    let days_in_month = next_month_date
        .signed_duration_since(first_day)
        .num_days() as u32;

    // 3. 1日の曜日オフセットを取得 (月曜=0, 火曜=1, ..., 日曜=6)
    // 月曜始まりのカレンダーにおける「第1週の空白の数」
    let start_offset = first_day.weekday().num_days_from_monday();

    // 4. 週数を計算
    // (日数 + オフセット) を 7 で割り、端数を切り上げる
    let total_cells = days_in_month + start_offset;
    (total_cells + 6) / 7
}

impl Guest for Component{
    type ShiftManager = AppState;
}

export!(Component);
