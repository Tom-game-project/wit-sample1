// --- カレンダー管理層の実装 ---

pub type LogicalDelta = usize;

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

/// 確定した予定を入れます
pub struct ShiftCalendarManager {
    base_abs_week: AbsWeek,
    initial_delta: LogicalDelta,

    timeline: Vec<WeekStatus>, // 実週番号 -> 状態
}

impl ShiftCalendarManager {
    pub fn new(base_abs_week: AbsWeek, initial_delta: LogicalDelta) -> Self {
        Self { base_abs_week, initial_delta, timeline: Vec::new() }
    }

    /// すでに作成したリストのなかにtarget_abs_weekを含み
    /// かつそれが変更される場合はエラーを返却する.
    pub fn append_week_with_check(
        &mut self, target_abs_week: AbsWeek,
        skip_flags: &[bool]
    ) -> Result<(), ()>{
        if let Some(latest_delta) = self.find_last_active_delta() {
            if  target_abs_week <= latest_delta {
                // 作成しようとしたシフトが既存のシフトを上書きしない
                for (index, is_skipped) in skip_flags.iter().enumerate() {
                    if let Some(content) = self.timeline.get(target_abs_week + index) {
                        if matches!(content, WeekStatus::Skipped) != *is_skipped { 
                            // 上書きを試みようとした
                            return Err(());
                        }
                    } else {
                        self.append_week(*is_skipped);
                    }
                }
                Ok(())
            }
            else if latest_delta + 1 == target_abs_week {
                for is_skipped in skip_flags {
                    self.append_week(*is_skipped);
                }
                Ok(())
            }
            else {
                // 生成しようとしている週のシフトが,保存されている最新より大きく決定しない週が存在してしまう
                // error
                Err(())
            }
        } else {
            for is_skipped in skip_flags {
                self.append_week(*is_skipped);
            }
            Ok(())
        }
    }

    fn append_week(
        &mut self,
        is_skipped: bool,
    ) {
        // 1. 次のdeltaを計算
        let next_delta = match self.timeline.last() {
            Some(
                WeekStatus::Active { logical_delta}) => logical_delta + 1,
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
