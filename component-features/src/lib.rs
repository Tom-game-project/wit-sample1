mod bindings {
    wit_bindgen::generate!({
        world: "my-world",
        generate_all
    });

    struct Component {}

    use exports::component::component_features::example_resource::{ 
        ShiftWeekday, 
        ShiftTime,
        GuestExampleList,
        Guest
    };

    use std::{
        cell::RefCell,
        collections::BTreeMap, ops::Add,
    };

    // --------------------------------------------------------
    // 1. Staff Groups Definition
    // --------------------------------------------------------
    //
    // スタッフ名を格納するスロット
    struct StaffGroup {
        name: String,         // 例: "Kitchen"
        slots: Vec<StaffInfo>,   // 例: ["Leader", "Sub", ""]
    }

    struct StaffInfo(String);

    impl StaffGroup {
        fn add_slot(&mut self) {
            self.slots.push(StaffInfo(String::from("")));
        }

        fn remove_slot(&mut self, slot_idx: u32) {
            self.slots.remove(slot_idx as usize);
        }

        fn update_memo(&mut self, staff_slot_index: u32, name:String) {
            if let Some(a) = self.slots.get_mut(staff_slot_index as usize) {
                a.0 = name;
            }
        }
    }

    // --------------------------------------------------------
    // 2. Weekly Rules Definition
    // --------------------------------------------------------
    // "a0", "b1" などのIDは String として扱います
    //
    // 週ごとのシフトホール格納スロット
    struct WeeklyRule {
        name: String,         // 例: "Standard Week"
        schedule: WeekSchedule
    }

    struct WeekSchedule {
        mon: DayShiftIds,
        tue: DayShiftIds,
        wed: DayShiftIds,
        thu: DayShiftIds,
        fri: DayShiftIds,
        sat: DayShiftIds,
        sun: DayShiftIds,
    }

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
            let day = match day {
                ShiftWeekday::Mon => &mut self.mon,
                ShiftWeekday::Tue => &mut self.tue,
                ShiftWeekday::Wed => &mut self.wed,
                ShiftWeekday::Thu => &mut self.thu,
                ShiftWeekday::Fri => &mut self.fri,
                ShiftWeekday::Sat => &mut self.sat,
                ShiftWeekday::Sun => &mut self.sun,
            };

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
            let day = match day {
                ShiftWeekday::Mon => &mut self.mon,
                ShiftWeekday::Tue => &mut self.tue,
                ShiftWeekday::Wed => &mut self.wed,
                ShiftWeekday::Thu => &mut self.thu,
                ShiftWeekday::Fri => &mut self.fri,
                ShiftWeekday::Sat => &mut self.sat,
                ShiftWeekday::Sun => &mut self.sun,
            };

            match shift_time {
                ShiftTime::Morning => {
                    day.m.remove(index as usize);
                }
                ShiftTime::Afternoon => {
                    day.a.remove(index as usize);
                } 
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
    struct DayShiftIds {
        m: Vec<Holl>,       // 午前シフトのIDリスト ["a0", "b1"]
        a: Vec<Holl>,       // 午後シフトのIDリスト []
    }

    impl DayShiftIds {
        fn new() -> Self {
            Self { m: vec![], a: vec![] }
        }
    }

    // シフトホール
    struct Holl {
        staff_group_id:u32, // スタッフグループを指す
        shift_staff_index:u32, // シフトのルールを司るindex
    }

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

    impl GuestExampleList for AppState {
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
            self.staff_groups.borrow_mut().push(
                StaffGroup { 
                    name: format!("Group{}", self.staff_groups.borrow().len()),
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
    }

    impl Guest for Component{
        type ExampleList = AppState;
    }

    export!(Component);
}

mod bindings2 {
    wit_bindgen::generate!({
        world: "my-world2",
        generate_all
    });

    struct Component {}

    use std::collections::HashMap;
    use dummy::logger::logger::log;
    use config_mock::config_mock::config_mock::{
        get_staff_groups, 
        get_week_rules
    };

    use chrono::{NaiveDate, Duration};
    use shift_calendar::{
        self,
        shift_gen::{
            DayRule, Incomplete, ShiftHoll, StaffGroup, StaffGroupList, WeekRule, WeekRuleTable
        }
    };

    fn calculate_weeks_delta_from_base(year: i32, month: u32, day: u32) -> Option<i64> {
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
        let date1 = NaiveDate::from_ymd_opt(1969, 12, 29)
            .unwrap() /* safe unwrap */;

        if let Some(date2)  = NaiveDate::from_ymd_opt(year, month, day) {
            let diff: Duration = date2 - date1;
            let weeks = diff.num_weeks();

            Some(weeks)
        } else {
            None
        }

    }

    fn day_shift<'a>(day_shift: 
        crate::bindings2::config_mock::config_mock::config_mock::DayShift) 
    -> DayRule<'a, Incomplete> {
        DayRule {
            shift_morning: day_shift.morning.iter().map(|i|
                ShiftHoll::new(i.group_id as usize, i.index as usize)
            ).collect(),
                shift_afternoon: day_shift.afternoon.iter().map(|i|
                ShiftHoll::new(i.group_id as usize, i.index as usize)
            ).collect(),
        }
    }

    impl Guest for Component {
        fn to_upper(input:String) -> String {
            let mut a:HashMap<String, String> = HashMap::new();

            a.insert(input.clone(), input.to_uppercase().clone());
            log(&format!("{:?}", a));

            log(&format!("経過週数: {}", 
                calculate_weeks_delta_from_base(1970, 1, 5).unwrap()
            ));
            log(&format!("経過週数: {}", 
                calculate_weeks_delta_from_base(1970, 1, 11).unwrap()
            ));
            log(&format!("経過週数: {}", 
                calculate_weeks_delta_from_base(1970, 1, 12).unwrap()
            ));

            // === 必要なデータの取得 ===

            // スタッフリストを取得する
            let staff_groups_form = get_staff_groups();
            // ルールを取得する
            let rules = get_week_rules();

            // === データの格納 ===

            // ロジックに渡せるようにデータを整える
            let mut staff_group_list = StaffGroupList::new();
            for i in &staff_groups_form {
                let mut staff_group = StaffGroup::new(&i.name);

                for name in &i.staff_list {
                    staff_group.add_staff(&name.name);
                }
                staff_group_list.add_staff_group(staff_group);
            }

            // ロジックに渡せるようにデータを整える
            let mut week_rule_table = WeekRuleTable::new();
            for i in rules {
                let week_rule = WeekRule([
                    day_shift(i.mon),
                    day_shift(i.tue),
                    day_shift(i.wed),
                    day_shift(i.thu),
                    day_shift(i.fri),
                    day_shift(i.sat),
                    day_shift(i.sun),
                ]);
                week_rule_table.add_week_rule(week_rule);
            }

            input.to_uppercase()
        }
    }

    export!(Component);
}
