// 想定通り生成したシフトを保存できているかを確認する

#[cfg(test)]
mod shift_calendar_manager_test {
    use component_features::shift_calendar_manager::{
        ShiftCalendarManager, 
        AppendWeekErrorKind
    };


    /// timelineがからのときの追加
    #[test]
    fn test00() {
        let mut shift_calendar_manager 
            = ShiftCalendarManager::new(2000, 0);
        // shift_calendar_manager.append_week(false); // 2000
        // shift_calendar_manager.append_week(false); // 2001
        // shift_calendar_manager.append_week(false); // 2002

        let r = shift_calendar_manager
            .append_check(
            2003,
            &[
                false, // 2001
                false, // 2002
                true   // 2003
            ]);
        assert!(matches!(r, Err(AppendWeekErrorKind::NotConsecutiveShifts)));
        // println!("{:?}", r);

    }

    #[test]
    fn test01() {
        let mut shift_calendar_manager 
            = ShiftCalendarManager::new(2000, 0);
        // shift_calendar_manager.append_week(false); // 2000
        // shift_calendar_manager.append_week(false); // 2001
        // shift_calendar_manager.append_week(false); // 2002

        let r = shift_calendar_manager
            .append_check(
            2003,
            &[
                false, // 2001
                false, // 2002
                true   // 2003
            ]);
        println!("{:?}", r);
    }
}

