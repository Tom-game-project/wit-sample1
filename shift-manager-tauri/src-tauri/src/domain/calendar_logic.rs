use crate::domain::shift_calendar_model::{
    AbsWeek, 
    ShiftCalendarManager, 
    WeekStatus,
    LogicalDelta,
    RuleId,
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
        rule_map: &HashMap<RuleId, WeekRuleTable<'a, Incomplete>>, // ID -> Rule
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
    rule_map: &HashMap<RuleId, WeekRuleTable<'a, Incomplete>>, // ID -> Rule
    staff_group_list: &'a StaffGroupList,
) -> Vec<Option<WeekDecidedShift<'a>>> {
    timeline_slice
        .iter().map(|i|{
            if let WeekStatus::Active { logical_delta , rule_id} = i {
                rule_map
                    .get(rule_id)
                    .map(|week_rule_table| 
                        gen_one_week_shift(
                            week_rule_table, 
                            staff_group_list,
                            *logical_delta
                        )
                    )
            } else {
                None
            }
        }).collect()
}

