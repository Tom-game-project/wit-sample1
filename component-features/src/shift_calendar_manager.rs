// --- カレンダー管理層の実装 ---

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
}

impl ShiftCalendarManager {
    pub fn new(base_abs_week: AbsWeek, initial_delta: LogicalDelta) -> Self {
        Self { base_abs_week, initial_delta, timeline: Vec::new() }
    }

    fn delta_to_abs(&self, delta: LogicalDelta) -> AbsWeek {
        let abs_week = delta + (self.base_abs_week - self.initial_delta);

        abs_week
    }

    fn abs_to_delta(&self, abs_week: AbsWeek) -> LogicalDelta {
        let delta = abs_week - (self.base_abs_week - self.initial_delta);

        delta
    }

    fn delta_to_index(&self, delta: LogicalDelta) -> usize {
        delta - self.initial_delta
    }

    fn abs_to_index(&self, abs_week: AbsWeek) -> usize {
        self.delta_to_index(self.abs_to_delta(abs_week))
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
        if self.timeline.len() < self.abs_to_delta(target_abs_week) {
            return Err(AppendWeekErrorKind::NotConsecutiveShifts);
        }

        if self
            .timeline[self.abs_to_index(target_abs_week)..]
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
}
