use crate::domain::shift_calendar_model::{
    AbsWeek, 
    ShiftCalendarManager, 
    WeekStatus,
    LogicalDelta,
    RuleID,
};

use shift_calendar::shift_gen::{
    gen_one_week_shift,
    WeekRuleTable,
    WeekDecidedShift,
    StaffGroupList,
    Incomplete
};

// TODO このエラーを本当にここに置くべきか
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
    fn abs_to_index(
        &self, 
        abs_week: AbsWeek
    ) -> Result<usize, AppendWeekErrorKind> {
        // self.delta_to_index(self.abs_to_delta(abs_week)?)
        if abs_week < self.base_abs_week {
            Err(AppendWeekErrorKind::UnderFlow)
        } else {
            Ok(abs_week - self.base_abs_week)
        }
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
    pub fn append_check<I: IntoIterator<Item = bool>>(&self, target_abs_week: AbsWeek, skip_flags: I) 
    -> Result<(), AppendWeekErrorKind>{
        if self.timeline.len() + self.base_abs_week < target_abs_week {
            return Err(AppendWeekErrorKind::NotConsecutiveShifts);
        }

        if self
            .timeline[self.abs_to_index(target_abs_week)?..]
            .iter()
            .zip(
                skip_flags
            ).all(|(week_status, is_skipped)|
                matches!(week_status, WeekStatus::Skipped) == is_skipped
            )
        {
            // Ok
        } else {
            return Err(AppendWeekErrorKind::AttemptedToOverwrite);
        }
        Ok(())
    }

    /// checkをしてOkであればtimelineに適用する
    /// self.append_check関数のチェックが入る
    pub fn apply_weeks(
        &mut self,
        target_abs_week: AbsWeek,
        skip_flags: &[(bool, RuleID)]
    ) -> Result<(), AppendWeekErrorKind> {

        self.append_check(target_abs_week, skip_flags.iter().map(|(a, _)| *a))?; // チェックをする

        // target_abs_week: 2,
        //  0, 1, 2  3  4  5  <- index
        // [T, F, F]          <- timeline     .len() -> 3
        //       [F, T, F, F] <- skip_flags
        //         [ T, F, F] <- append_skip_flags
        // 
        //  let after_timeline_len = target_abs_week (2) + skip_flags.len() (4);
        //  let append_len = after_timeline_len (6) - self.timeline.len() (3); (3)
        //  let append_start_index = skip_flags.len() (4) - append_len (3); (1)     // append start index
        //  let append_skip_flags = [append_start_index..]
        //  [T, T, T, T]
        //  [T, T]      
        //              []
        //
        if self.timeline.len() < self.abs_to_index(target_abs_week)? {
            return Err(AppendWeekErrorKind::UnderFlow);
        }

        let append_start_index = self.timeline.len() - self.abs_to_index(target_abs_week)?;

        // 重要: indexを超えている場合
        // 何もしない
        if skip_flags.len() <= append_start_index {
            return Ok(());
        }

        for (is_skipped, rule_id) in &skip_flags[append_start_index..] {
            self.append_week(*is_skipped, *rule_id);
        }
        Ok(())
    }
    
    pub fn append_week(
        &mut self,
        is_skipped: bool, rule_id: RuleID
    ) {
        // 1. 次のdeltaを計算
        let next_delta = match self.timeline.last() {
            Some(WeekStatus::Active { logical_delta, rule_id:_ }) => logical_delta + 1,
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
                logical_delta: next_delta, rule_id
            }
        };

        self.timeline.push(slot);
    }

    /// 直近の有効なDeltaを探すヘルパー
    fn find_last_active_delta(&self) -> Option<LogicalDelta> {
        self.timeline.iter().rev().find_map(|slot| match slot {
            WeekStatus::Active {
                logical_delta, rule_id:_
            } => Some(*logical_delta),
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

    /// シフトの導出
    /// base_abs_weekを下回る場合でも返せる場合があるが
    /// skipを使うことで週の途中からルールを開始することは可能なので対応しない
    pub fn derive_shift<'a>(
        &self,
        rule_map: &HashMap<RuleID, WeekRuleTable<'a, Incomplete>>, // ID -> Rule
        staff_group_list: &'a StaffGroupList,
        gen_week_abs: AbsWeek,  // 生成の始点となる絶対週
        gen_range: usize,       // 何週間分のシフトを作成するか
    ) -> Vec<Option<WeekDecidedShift<'a>>>{
        if let Ok(index) = self.abs_to_index(gen_week_abs) {
            if index + gen_range < self.timeline.len() {
                calculate_partial_shift(
    &self.timeline[
                        index..index + gen_range
                    ], rule_map, staff_group_list
                )
            } else if index < self.timeline.len() {
                calculate_partial_shift(
                    &self.timeline[
                    index..
                ], 
                rule_map,
                staff_group_list)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
}

use std::collections::HashMap;

/// 指定された期間のシフトのみを計算する純粋関数
///
/// - `timeline_slice`: 計算対象の週のステータス（例: 4週間分だけ）
/// - `rule_map`: rule_id から 実際のWeekRule へのマップ (必要な分だけ)
/// - `staff_groups`: スタッフリスト (これはサイズが小さいので全件でもOKだが、最適化も可能)
pub fn calculate_partial_shift<'a>(
    timeline_slice: &[WeekStatus],
    rule_map: &HashMap<RuleID, WeekRuleTable<'a, Incomplete>>, // ID -> Rule
    staff_group_list: &'a StaffGroupList,
) -> Vec<Option<WeekDecidedShift<'a>>> {
    timeline_slice
        .iter().map(|i|{
            if let WeekStatus::Active { logical_delta , rule_id} = i {
                if let Some(week_rule_table) = rule_map.get(rule_id) {
                    Some(
                        gen_one_week_shift(
                            week_rule_table, 
                            staff_group_list,
                            *logical_delta
                        )
                    )
                } else {
                    None
                }
            } else {
                None
            }
        }).collect()
}

