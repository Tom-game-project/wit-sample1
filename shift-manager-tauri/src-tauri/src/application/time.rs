use chrono::{Datelike, Duration, NaiveDate};
use crate::domain::shift_calendar_model::AbsWeek;

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
    let start_offset = first_day
        .weekday()
        .num_days_from_monday();

    // 4. 週数を計算
    // (日数 + オフセット) を 7 で割り、端数を切り上げる
    let total_cells = days_in_month + start_offset;
    (total_cells + 6) / 7
}

/// ヘルパー: 年月から絶対週番号を計算する (JS側と合わせる必要あり)
pub fn calculate_abs_week(year: i32, month: u32, day: u32) -> Option<AbsWeek>  {
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
