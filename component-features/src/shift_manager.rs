wit_bindgen::generate!({
    world: "shift-manager-world",
    generate_all
});

struct Component {}

use exports::component::component_features::shift_manager::{ 
    ShiftWeekday, 
    ShiftTime,
    GuestShiftManager,
    Guest,
    // staff group data
    StaffGroup,
    StaffInfo,
    // weekly rule data
    WeeklyRule,
    WeekSchedule,
    DayShiftIds,
    Holl,
};

use std::{
    cell::RefCell,
    collections::BTreeMap,
};

use crate::shift_gen::dummy::logger::logger::log;

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
    fn extract_day_shift_ids<'a>(&self, week_schedule: &'a WeekSchedule) -> &'a DayShiftIds{
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
}

// シフトホール
// struct Holl {
//     staff_group_id:u32, // スタッフグループを指す
//     shift_staff_index:u32, // シフトのルールを司るindex
// }

// --------------------------------------------------------
// 3. Calendar Data Definition
// --------------------------------------------------------
// カレンダーは ID("a0") ではなく、解決済みのオブジェクト(名前+色ID)を持ちます
struct CalendarEntry {
    m: ResolvedWeekly, // morning
    a: ResolvedWeekly, // afternoon
}

struct ResolvedWeekly {
    mon: ResolvedStaff,
    tue: ResolvedStaff,
    wed: ResolvedStaff,
    thu: ResolvedStaff,
    fri: ResolvedStaff,
    sat: ResolvedStaff,
    sun: ResolvedStaff,
}

struct ResolvedStaff {
    name: String,      // スタッフの名前
    staff_group_id: u32,    // スタッフグループのindex
    staff_slot_index: u32,  // 対象スタッフのスタッフグループ内でのindex
}

// --------------------------------------------------------
// 4. Root State (Entire Application State)
// --------------------------------------------------------
//
// この構造体がアプリ全体の状態を管理する
// このモジュールはwit/depsに依存する
struct AppState {
    staff_groups: RefCell<Vec<StaffGroup>>,
    rules: RefCell<Vec<WeeklyRule>>,

    year: RefCell<u32>,            // 例: 2026
    month: RefCell<u32>,           // 例: 0 (Jan) - 11 (Dec)

    // Key: "YYYY-MM-DD"
    // 実際に生成されたカレンダー
    schedule_data: RefCell<BTreeMap<String, CalendarEntry>>,
}

impl GuestShiftManager for AppState {
    fn new() -> Self {
        Self {
            staff_groups: RefCell::new(vec![]),
            rules: RefCell::new(vec![]),
            year: RefCell::new(2026), // TODO 初期値はdeps内の関数から取得する必要あり
            month: RefCell::new(1),   // TODO  初期値はdeps内の関数から取得する必要あり
            schedule_data: RefCell::new(BTreeMap::new())
        }
    }

    fn add_new_group(&self) {
        let staff_group_length = self.staff_groups.borrow().len();
        // log("add_new_group");
        self.staff_groups.borrow_mut().push(
            StaffGroup { 
                name: format!("Group{}", staff_group_length),
                slots: vec![]
            }
        );
        // log(&format!("{:?}", self.staff_groups));
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
            // log("add alot");
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

    fn add_rule(&self) {
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

    fn get_rules(&self,) -> Vec<WeeklyRule> {
        self.rules.borrow().clone()
    }

    fn get_staff_groups(&self,) -> Vec<StaffGroup> {
        self.staff_groups.borrow().clone()
    }

    fn get_year(&self,) -> u32 {
        *self.year.borrow()
    }

    fn get_month(&self,) -> u32 {
        *self.month.borrow()
    }
}

impl Guest for Component {
    type ShiftManager = AppState;
}

export!(Component);
