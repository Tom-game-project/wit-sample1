wit_bindgen::generate!({
    world: "shift-gen-world",
    generate_all
});

struct Component {}

use config_mock::config_mock::config_mock::{
    get_staff_groups, 
    get_week_rules
};

use chrono::{NaiveDate, Duration};
use shift_calendar::{
    self,
    shift_gen::{
        DayRule,
        Incomplete, 
        ShiftHoll, 
        StaffGroup, 
        StaffGroupList, 
        WeekRule, 
        WeekRuleTable
    }
};

use crate::shift_gen::{
    config_mock::config_mock::config_mock::DayShift,
    dummy::logger::logger::log
};

fn day_shift<'a>(day_shift: 
    DayShift
) -> DayRule<'a, Incomplete> {
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

        log(&format!("経過週数: {}", 
            Component::calculate_weeks_delta_from_base(1970, 1, 5).unwrap()
        ));
        log(&format!("経過週数: {}", 
            Component::calculate_weeks_delta_from_base(1970, 1, 11).unwrap()
        ));
        log(&format!("経過週数: {}", 
            Component::calculate_weeks_delta_from_base(1970, 1, 12).unwrap()
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

    fn calculate_weeks_delta_from_base(year:i32, month:u32, day:u32,) -> Option<i64> {
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

        if let Some(date2)  = NaiveDate::from_ymd_opt(year, month, day) {
            let diff: Duration = date2 - date1;
            let weeks = diff.num_weeks();

            if weeks < 0 {
                None
            } else {
                Some(weeks)
            }
            
        } else {
            None
        }
    }
}

export!(Component);

