// SPDX-License-Identifier: MIT
//! 雙擊 Shift 快速切換手勢判定器 (DoubleShiftGesture)
//! 移植自 edwardhome/AI-Bopomofo (DoubleShiftGesture.swift)
//!
//! 規則：
//! - 兩次獨立短按放開 Shift
//! - 每次按住時間不超過 250ms (tap_limit = 0.25s)
//! - 兩次放開間隔不超過 350ms (interval = 0.35s)
//! - 夾帶其他修飾鍵、打字或長按均自動取消手勢

use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct DoubleShiftGesture {
    pub tap_limit: Duration,
    pub interval: Duration,
    down_at: Option<Duration>,
    first_release: Option<Duration>,
    shift_was_down: bool,
}

impl Default for DoubleShiftGesture {
    fn default() -> Self {
        Self {
            tap_limit: Duration::from_millis(250),
            interval: Duration::from_millis(350),
            down_at: None,
            first_release: None,
            shift_was_down: false,
        }
    }
}

impl DoubleShiftGesture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.down_at = None;
        self.first_release = None;
        self.shift_was_down = false;
    }

    fn cancel_sequence(&mut self) {
        self.down_at = None;
        self.first_release = None;
    }

    /// 更新按鍵狀態。
    /// time: 自程式啟動以來的單調遞增時間 Duration
    /// 返回 true 表示成功觸發雙擊 Shift 切換中英
    pub fn update(&mut self, shift_down: bool, other_modifiers: bool, time: Duration) -> bool {
        let changed = shift_down != self.shift_was_down;
        self.shift_was_down = shift_down;

        if other_modifiers {
            self.cancel_sequence();
            return false;
        }

        if !changed {
            // 重複修飾鍵事件通知不重置計時器
            return false;
        }

        if shift_down {
            if let Some(last) = self.first_release {
                if time < last || time - last > self.interval {
                    self.first_release = None;
                }
            }
            self.down_at = Some(time);
            return false;
        }

        // Shift 放開事件
        let down_at = match self.down_at {
            Some(d) => d,
            None => {
                self.cancel_sequence();
                return false;
            }
        };

        if time < down_at || time - down_at > self.tap_limit {
            self.cancel_sequence();
            return false;
        }

        self.down_at = None;

        if let Some(last) = self.first_release {
            if time >= last && time - last <= self.interval {
                self.first_release = None;
                return true; // 觸發雙擊 Shift！
            }
        }

        self.first_release = Some(time);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_shift_success() {
        let mut gesture = DoubleShiftGesture::new();
        // 第一次點擊 Shift (50ms)
        assert!(!gesture.update(true, false, Duration::from_millis(100)));
        assert!(!gesture.update(false, false, Duration::from_millis(150)));

        // 間隔 100ms 後第二次點擊 Shift (50ms)
        assert!(!gesture.update(true, false, Duration::from_millis(250)));
        // 第二次放開，時間在 350ms 內 -> 觸發！
        assert!(gesture.update(false, false, Duration::from_millis(300)));
    }

    #[test]
    fn test_double_shift_timeout() {
        let mut gesture = DoubleShiftGesture::new();
        // 第一次點擊
        assert!(!gesture.update(true, false, Duration::from_millis(100)));
        assert!(!gesture.update(false, false, Duration::from_millis(150)));

        // 間隔 500ms (超過 350ms)
        assert!(!gesture.update(true, false, Duration::from_millis(650)));
        assert!(!gesture.update(false, false, Duration::from_millis(700)));
    }

    #[test]
    fn test_double_shift_interrupted_by_modifier() {
        let mut gesture = DoubleShiftGesture::new();
        assert!(!gesture.update(true, false, Duration::from_millis(100)));
        assert!(!gesture.update(false, false, Duration::from_millis(150)));

        // 夾帶 Ctrl
        assert!(!gesture.update(true, true, Duration::from_millis(200)));
        assert!(!gesture.update(false, false, Duration::from_millis(250)));
    }
}
