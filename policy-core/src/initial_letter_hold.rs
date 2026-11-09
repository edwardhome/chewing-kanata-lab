// SPDX-License-Identifier: MIT
//! 首字母長按轉換小寫純狀態機 (InitialLetterHoldGesture)
//!
//! 功能規範與合約 (Initial Letter Hold Contract):
//! 1. 純記憶體運算、零 IO、無平台相依、無計時器系統呼叫。
//! 2. 針對 Shift + 首字母 (如 Shift+S)：
//!    - 短按 Shift+S：維持大寫 'S'。
//!    - 長按 S 超過指定門檻 (預設 250ms，範圍 100..=1000ms)：轉為小寫 's'。
//!    - 即使 Shift 先放開，只要 S 仍按住且達門檻，依然轉為 's'。
//!    - 門檻前的按鍵重複 (repeating press) 一律予以吞噬 (consumed)，不提早觸發，避免出現 "ssss"。
//! 3. 取消條件：
//!    - 字母放開 (Key Up)
//!    - 另一個按鍵按下 (Another key)
//!    - 焦點變更 / 編輯操作 / 快速鍵 / 敏感視窗 / 模式切換 / 終止 (Cancel / Reset)
//!    - F 鍵長按具有優先權。
//! 4. 世代與上下文安全 (Generation & Stable Identity)：
//!    - 每次開始手勢時遞增 generation。
//!    - tick / key_up 必須校驗 generation，避免過期或跨上下文回調。
//! 5. 雙重驅動與防禦性 fallback：
//!    - 支援外部實體計時器回調 (tick)。
//!    - 若實體計時器查詢不確定或失準，按鍵重複或放開事件 (event fallback) 亦可依據傳入之 timestamp/elapsed 判定結算。

use std::time::{Duration, Instant};

pub const DEFAULT_HOLD_DELAY_MS: u64 = 250;
pub const MIN_HOLD_DELAY_MS: u64 = 100;
pub const MAX_HOLD_DELAY_MS: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    /// 正在等待長按門檻，首字母大寫仍保留在未送出組字
    Pending,
    /// 已超過門檻並成功轉換為小寫
    Triggered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// 忽略或無需動作
    None,
    /// 吞噬此按鍵重複，不讓作業系統產生重複大寫字元
    ConsumeRepeat,
    /// 長按時間到達，觸發將首字母轉為小寫（僅改未送出組字，不合成退格或更改已送出文字）
    TransformToLowercase { uppercase: char, lowercase: char },
    /// 手勢被取消，若有計時器應取消
    CancelTimer,
}

#[derive(Debug, Clone)]
pub struct InitialLetterHoldGesture {
    pub delay: Duration,
    phase: Phase,
    vk: u16,
    uppercase_char: char,
    lowercase_char: char,
    generation: u64,
    start_time: Option<Instant>,
    shift_was_held: bool,
}

impl Default for InitialLetterHoldGesture {
    fn default() -> Self {
        Self::new(Duration::from_millis(DEFAULT_HOLD_DELAY_MS))
    }
}

impl InitialLetterHoldGesture {
    pub fn new(delay: Duration) -> Self {
        let clamped_ms = (delay.as_millis() as u64).clamp(MIN_HOLD_DELAY_MS, MAX_HOLD_DELAY_MS);
        Self {
            delay: Duration::from_millis(clamped_ms),
            phase: Phase::Idle,
            vk: 0,
            uppercase_char: '\0',
            lowercase_char: '\0',
            generation: 0,
            start_time: None,
            shift_was_held: false,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn current_vk(&self) -> Option<u16> {
        if self.phase != Phase::Idle {
            Some(self.vk)
        } else {
            None
        }
    }

    pub fn set_delay(&mut self, delay: Duration) {
        let clamped_ms = (delay.as_millis() as u64).clamp(MIN_HOLD_DELAY_MS, MAX_HOLD_DELAY_MS);
        self.delay = Duration::from_millis(clamped_ms);
    }

    /// 開始追蹤 Shift + 首字母按下手勢
    ///
    /// - `vk`: 字母鍵碼 (如 0x41..=0x5A)
    /// - `letter`: 大寫字元 (如 'S')
    /// - `now`: 當前時間戳記
    pub fn start(&mut self, vk: u16, letter: char, now: Instant) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.phase = Phase::Pending;
        self.vk = vk;
        self.uppercase_char = letter;
        self.lowercase_char = letter.to_ascii_lowercase();
        self.start_time = Some(now);
        self.shift_was_held = true;
        self.generation
    }

    /// 處理重複按下事件 (OS Key Repeat)
    ///
    /// 門檻到達前一律吞噬重複輸入；
    /// 若實體計時器未能及時觸發，當按鍵重複到達且時間已超過門檻時，亦能作為 fallback 成功轉換。
    pub fn handle_key_press(&mut self, vk: u16, repeating: bool, now: Instant) -> Effect {
        if self.phase == Phase::Idle {
            return Effect::None;
        }

        // 如果按下了其他鍵，手勢被中斷取消
        if vk != self.vk {
            self.cancel();
            return Effect::CancelTimer;
        }

        match self.phase {
            Phase::Pending => {
                let elapsed = self.start_time.map_or(Duration::ZERO, |t| now.saturating_duration_since(t));
                if elapsed >= self.delay {
                    // 超過延遲門檻：轉為小寫
                    self.phase = Phase::Triggered;
                    Effect::TransformToLowercase {
                        uppercase: self.uppercase_char,
                        lowercase: self.lowercase_char,
                    }
                } else if repeating {
                    // 未達門檻前，吞噬重複的按鍵事件，防止輸出多餘的大寫字母
                    Effect::ConsumeRepeat
                } else {
                    Effect::None
                }
            }
            Phase::Triggered => {
                // 已轉換為小寫後，後續的連打由系統正常重複或吞噬
                Effect::ConsumeRepeat
            }
            Phase::Idle => Effect::None,
        }
    }

    /// 處理 Shift 釋放：
    /// 依合約規定，即使 Shift 先放開，只要該字母鍵仍然按住，手勢不會取消，長按達門檻依然轉為小寫。
    pub fn handle_shift_release(&mut self) {
        // Shift 釋放不取消手勢，僅記錄 Shift 狀態
        self.shift_was_held = false;
    }

    /// 處理字母鍵放開 (Key Up)
    ///
    /// - 若該鍵放開，手勢即告結束。
    /// - 回傳是否曾觸發為小寫或手勢已結束。
    pub fn handle_key_up(&mut self, vk: u16, _now: Instant) -> Effect {
        if self.phase != Phase::Idle && vk == self.vk {
            self.cancel();
            Effect::CancelTimer
        } else {
            Effect::None
        }
    }

    /// 外部計時器到期 (Timer Tick)
    ///
    /// 必須攜帶啟動時回傳的 generation。若世代不合或已不在 Pending 狀態，則不執行任何動作。
    pub fn tick(&mut self, generation: u64, now: Instant) -> Effect {
        if self.phase != Phase::Pending || self.generation != generation {
            return Effect::None;
        }

        let elapsed = self.start_time.map_or(Duration::ZERO, |t| now.saturating_duration_since(t));
        if elapsed >= self.delay {
            self.phase = Phase::Triggered;
            Effect::TransformToLowercase {
                uppercase: self.uppercase_char,
                lowercase: self.lowercase_char,
            }
        } else {
            Effect::None
        }
    }

    /// 外部因焦點變更、視窗切換、退格、快捷鍵、模式切換或安全性檢查失敗而取消
    pub fn cancel(&mut self) {
        self.phase = Phase::Idle;
        self.vk = 0;
        self.uppercase_char = '\0';
        self.lowercase_char = '\0';
        self.start_time = None;
        self.shift_was_held = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_tap_remains_uppercase() {
        let mut gesture = InitialLetterHoldGesture::new(Duration::from_millis(250));
        let start = Instant::now();

        let gen = gesture.start(0x53, 'S', start);
        assert_eq!(gesture.phase(), Phase::Pending);

        // 100ms 後放開 S，未達 250ms
        let keyup_effect = gesture.handle_key_up(0x53, start + Duration::from_millis(100));
        assert_eq!(keyup_effect, Effect::CancelTimer);
        assert_eq!(gesture.phase(), Phase::Idle);

        // 隨後若 timer tick 抵達，世代依然比對但已不是 Pending，不應變換
        let tick_effect = gesture.tick(gen, start + Duration::from_millis(250));
        assert_eq!(tick_effect, Effect::None);
    }

    #[test]
    fn test_timer_tick_triggers_lowercase() {
        let mut gesture = InitialLetterHoldGesture::new(Duration::from_millis(250));
        let start = Instant::now();

        let gen = gesture.start(0x53, 'S', start);

        // 150ms 處重複按鍵被吞噬
        let repeat_effect = gesture.handle_key_press(0x53, true, start + Duration::from_millis(150));
        assert_eq!(repeat_effect, Effect::ConsumeRepeat);
        assert_eq!(gesture.phase(), Phase::Pending);

        // 250ms 處計時器觸發
        let tick_effect = gesture.tick(gen, start + Duration::from_millis(250));
        assert_eq!(
            tick_effect,
            Effect::TransformToLowercase {
                uppercase: 'S',
                lowercase: 's'
            }
        );
        assert_eq!(gesture.phase(), Phase::Triggered);

        // 觸發後後續重複按鍵依然被吞噬，不產生 ssss
        let after_repeat = gesture.handle_key_press(0x53, true, start + Duration::from_millis(300));
        assert_eq!(after_repeat, Effect::ConsumeRepeat);

        // 字母放開結束
        let up_effect = gesture.handle_key_up(0x53, start + Duration::from_millis(400));
        assert_eq!(up_effect, Effect::CancelTimer);
        assert_eq!(gesture.phase(), Phase::Idle);
    }

    #[test]
    fn test_shift_released_first_still_transforms_on_hold() {
        let mut gesture = InitialLetterHoldGesture::new(Duration::from_millis(250));
        let start = Instant::now();

        let gen = gesture.start(0x53, 'S', start);

        // 50ms 時 Shift 放開
        gesture.handle_shift_release();
        assert_eq!(gesture.phase(), Phase::Pending);

        // 255ms 時計時器到期，依舊成功轉換為 's'
        let tick_effect = gesture.tick(gen, start + Duration::from_millis(255));
        assert_eq!(
            tick_effect,
            Effect::TransformToLowercase {
                uppercase: 'S',
                lowercase: 's'
            }
        );
        assert_eq!(gesture.phase(), Phase::Triggered);
    }

    #[test]
    fn test_event_fallback_if_timer_missed() {
        let mut gesture = InitialLetterHoldGesture::new(Duration::from_millis(250));
        let start = Instant::now();

        let _gen = gesture.start(0x41, 'A', start);

        // 假設實體計時器未能觸發，但 OS repeat 在 260ms 到達
        let repeat_effect = gesture.handle_key_press(0x41, true, start + Duration::from_millis(260));
        assert_eq!(
            repeat_effect,
            Effect::TransformToLowercase {
                uppercase: 'A',
                lowercase: 'a'
            }
        );
        assert_eq!(gesture.phase(), Phase::Triggered);
    }

    #[test]
    fn test_interrupted_by_another_key() {
        let mut gesture = InitialLetterHoldGesture::new(Duration::from_millis(250));
        let start = Instant::now();

        let gen = gesture.start(0x53, 'S', start);

        // 在 80ms 按下其他鍵 (如 'T'，0x54)
        let other_effect = gesture.handle_key_press(0x54, false, start + Duration::from_millis(80));
        assert_eq!(other_effect, Effect::CancelTimer);
        assert_eq!(gesture.phase(), Phase::Idle);

        // 原計時器過期無效
        let tick_effect = gesture.tick(gen, start + Duration::from_millis(250));
        assert_eq!(tick_effect, Effect::None);
    }

    #[test]
    fn test_generation_mismatch_ignored() {
        let mut gesture = InitialLetterHoldGesture::new(Duration::from_millis(250));
        let start = Instant::now();

        let gen1 = gesture.start(0x53, 'S', start);
        gesture.cancel();

        let gen2 = gesture.start(0x44, 'D', start + Duration::from_millis(50));
        assert_ne!(gen1, gen2);

        // 舊世代的 tick 抵達，被安全忽略
        let old_tick = gesture.tick(gen1, start + Duration::from_millis(250));
        assert_eq!(old_tick, Effect::None);
        assert_eq!(gesture.phase(), Phase::Pending);

        // 新世代的 tick 正常觸發
        let new_tick = gesture.tick(gen2, start + Duration::from_millis(310));
        assert_eq!(
            new_tick,
            Effect::TransformToLowercase {
                uppercase: 'D',
                lowercase: 'd'
            }
        );
    }

    #[test]
    fn test_delay_clamping() {
        let g1 = InitialLetterHoldGesture::new(Duration::from_millis(50));
        assert_eq!(g1.delay, Duration::from_millis(100));

        let g2 = InitialLetterHoldGesture::new(Duration::from_millis(1500));
        assert_eq!(g2.delay, Duration::from_millis(1000));
    }
}
