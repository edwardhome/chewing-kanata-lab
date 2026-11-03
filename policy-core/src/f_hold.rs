// SPDX-License-Identifier: MIT
//! F 鍵長按與分層手勢判定器 (FHoldGesture)
//! 移植自 edwardhome/AI-Bopomofo (QuickSymbolLayer.swift 中的 FHoldGesture 與 canStartChord)
//!
//! 功能說明：
//! 1. 支援 F 鍵長按進入符號數字分層 (held >= 150ms/180ms)
//! 2. 支援 F 鍵未放開時的立即組合鍵 (0ms 即時 chording，如 F+R -> +, F+Esc -> `)
//! 3. 快速注音保護 (0.6.11 fix/f-phonetic-overlap)：
//!    在標準注音鍵盤下，F (ㄑ) 在長按門檻內 (pending) 遭遇 U (ㄧ) 或 M (ㄩ) 時，
//!    優先視為注音音節 (ㄑㄧ / ㄑㄩ) 而非數字 8 / 2。
//!    此時取消分層，先還原 F (ㄑ)，再處理 U/M，避免高速打字時誤輸出數字。
//!    若需輸入數字 8 / 2，需先長按 F 超過門檻 (held) 再按對應鍵。

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Pending,
    Held,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Press { eligible: bool, repeating: bool },
    Release,
    ObservedRelease,
    OtherKey { is_mapped: bool },
    Timeout,
    Cancel,
    Elapsed(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Schedule,
    Invalidate,
    ReplayTap,
    BeginLayer,
    EndLayer,
    Consume,
}

#[derive(Debug, Clone)]
pub struct FHoldGesture {
    pub delay: Duration,
    pub phase: Phase,
    owns_key_up: bool,
}

impl Default for FHoldGesture {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(Self::DEFAULT_DELAY_MS),
            phase: Phase::Idle,
            owns_key_up: false,
        }
    }
}

impl FHoldGesture {
    pub const DEFAULT_DELAY_MS: u64 = 150;
    pub const MIN_DELAY_MS: u64 = 100;
    pub const MAX_DELAY_MS: u64 = 1000;

    pub fn new(delay: Duration) -> Self {
        let clamped_ms = (delay.as_millis() as u64).clamp(Self::MIN_DELAY_MS, Self::MAX_DELAY_MS);
        Self {
            delay: Duration::from_millis(clamped_ms),
            phase: Phase::Idle,
            owns_key_up: false,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn owns_key_up(&self) -> bool {
        self.owns_key_up
    }

    pub fn handle(&mut self, event: Event) -> Vec<Effect> {
        match event {
            Event::Press { eligible, repeating } => {
                if self.phase != Phase::Idle || (repeating && self.owns_key_up) {
                    return vec![Effect::Consume];
                }
                if !eligible || repeating {
                    return vec![];
                }
                self.phase = Phase::Pending;
                self.owns_key_up = true;
                vec![Effect::Schedule, Effect::Consume]
            }
            Event::ObservedRelease => {
                let owned = self.owns_key_up;
                let effects = self.handle(Event::Release);
                self.owns_key_up = owned;
                effects
            }
            Event::Release => {
                let owned = self.owns_key_up;
                self.owns_key_up = false;
                let previous = self.phase;
                self.phase = Phase::Idle;
                match previous {
                    Phase::Pending => vec![Effect::Invalidate, Effect::ReplayTap, Effect::Consume],
                    Phase::Held => vec![Effect::Invalidate, Effect::EndLayer, Effect::Consume],
                    Phase::Idle => {
                        if owned {
                            vec![Effect::Consume]
                        } else {
                            vec![]
                        }
                    }
                }
            }
            Event::OtherKey { is_mapped } => {
                if self.phase != Phase::Pending {
                    return vec![];
                }
                if is_mapped {
                    self.handle(Event::Timeout)
                } else {
                    self.phase = Phase::Idle;
                    vec![Effect::Invalidate, Effect::ReplayTap]
                }
            }
            Event::Elapsed(duration) => {
                if self.phase == Phase::Pending && duration > self.delay {
                    self.handle(Event::Timeout)
                } else {
                    vec![]
                }
            }
            Event::Timeout => {
                if self.phase != Phase::Pending {
                    return vec![];
                }
                self.phase = Phase::Held;
                vec![Effect::BeginLayer]
            }
            Event::Cancel => {
                self.owns_key_up = false;
                let previous = self.phase;
                self.phase = Phase::Idle;
                if previous == Phase::Held {
                    vec![Effect::Invalidate, Effect::EndLayer]
                } else {
                    vec![Effect::Invalidate]
                }
            }
        }
    }
}

/// 判斷在標準注音鍵盤配置下，是否為與 F (ㄑ) 產生合法音節重疊之韻母鍵位 (U -> ㄧ, M -> ㄩ)
pub fn is_phonetic_f_overlap(vk: u16) -> bool {
    // 0x55: VK_U ('u' / ㄧ)
    // 0x4D: VK_M ('m' / ㄩ)
    vk == 0x55 || vk == 0x4D
}

/// 判斷在目前狀態下是否應優先保留注音 (ㄑㄧ / ㄑㄩ) 而非觸發 F 立即組合鍵 (8 / 2)
/// 移植自 KeyHandler.mm 之 prefersPhoneticFChord
pub fn prefers_phonetic_f_chord(
    vk: u16,
    is_pending: bool,
    is_chinese_mode: bool,
    is_default_layout: bool,
    has_modifiers: bool,
) -> bool {
    is_pending && is_chinese_mode && is_default_layout && !has_modifiers && is_phonetic_f_overlap(vk)
}

/// 判斷是否能啟動 F 組合鍵 (對齊 QuickSymbolLayer.canStartChord)
pub fn can_start_chord(
    has_output: bool,
    repeating: bool,
    has_disallowed_modifiers: bool,
    prefers_phonetic: bool,
) -> bool {
    !prefers_phonetic && !repeating && !has_disallowed_modifiers && has_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phonetic_overlap_replays_once_and_never_reenters() {
        for &vk in &[0x55u16, 0x4Du16] { // U (8 / ㄧ), M (2 / ㄩ)
            for &ms in &[100u64, 150u64, 250u64, 1000u64] {
                let delay = Duration::from_millis(ms);
                for &elapsed_ms in &[1u64, ms.saturating_sub(1), ms] {
                    for f_released_first in [false, true] {
                        let mut gesture = FHoldGesture::new(delay);
                        let effects = gesture.handle(Event::Press { eligible: true, repeating: false });
                        assert_eq!(effects, vec![Effect::Schedule, Effect::Consume]);

                        let elapsed = gesture.handle(Event::Elapsed(Duration::from_millis(elapsed_ms)));
                        assert!(elapsed.is_empty());

                        // 判斷注音優先
                        let prefer = prefers_phonetic_f_chord(vk, gesture.phase() == Phase::Pending, true, true, false);
                        assert!(prefer);

                        let mapped = can_start_chord(true, false, false, prefer);
                        assert!(!mapped);

                        let chord_effects = gesture.handle(Event::OtherKey { is_mapped: mapped });
                        assert_eq!(chord_effects, vec![Effect::Invalidate, Effect::ReplayTap]);
                        assert_eq!(gesture.phase(), Phase::Idle);

                        // 已選注音就不能被晚到的計時器或仍按住的 F 重複事件變回符號
                        assert!(gesture.handle(Event::Elapsed(delay + Duration::from_millis(100))).is_empty());
                        assert!(gesture.handle(Event::Timeout).is_empty());
                        assert_eq!(
                            gesture.handle(Event::Press { eligible: true, repeating: true }),
                            vec![Effect::Consume]
                        );
                        assert!(gesture.handle(Event::OtherKey { is_mapped: true }).is_empty());

                        if f_released_first {
                            assert_eq!(gesture.handle(Event::Release), vec![Effect::Consume]);
                        } else {
                            assert_eq!(gesture.handle(Event::ObservedRelease), vec![Effect::Consume]);
                            assert_eq!(gesture.handle(Event::Release), vec![Effect::Consume]);
                        }
                        assert!(gesture.handle(Event::Release).is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn test_long_hold_keeps_eight_and_two_after_phonetic_deadline() {
        for &vk in &[0x55u16, 0x4Du16] {
            let mut gesture = FHoldGesture::new(Duration::from_millis(150));
            gesture.handle(Event::Press { eligible: true, repeating: false });

            // 超過 150ms 門檻，進入 Held
            let effects = gesture.handle(Event::Elapsed(Duration::from_millis(151)));
            assert_eq!(effects, vec![Effect::BeginLayer]);
            assert_eq!(gesture.phase(), Phase::Held);

            // 長按後已非 pending，不觸發注音優先
            let prefer = prefers_phonetic_f_chord(vk, gesture.phase() == Phase::Pending, true, true, false);
            assert!(!prefer);

            let mapped = can_start_chord(true, false, false, prefer);
            assert!(mapped);

            // 放開 F
            assert_eq!(
                gesture.handle(Event::Release),
                vec![Effect::Invalidate, Effect::EndLayer, Effect::Consume]
            );
            assert_eq!(gesture.phase(), Phase::Idle);
        }
    }

    #[test]
    fn test_english_and_other_chords_keep_immediate_entry() {
        // R (0x52 -> +), Esc (0x1B -> `), Y (0x59 -> 7)
        for &vk in &[0x52u16, 0x1Bu16, 0x59u16] {
            let mut gesture = FHoldGesture::new(Duration::from_millis(250));
            gesture.handle(Event::Press { eligible: true, repeating: false });

            // 其他鍵不受注音衝突保護，立即 chording
            let prefer = prefers_phonetic_f_chord(vk, gesture.phase() == Phase::Pending, true, true, false);
            assert!(!prefer);

            let mapped = can_start_chord(true, false, false, prefer);
            assert!(mapped);

            assert_eq!(gesture.handle(Event::OtherKey { is_mapped: mapped }), vec![Effect::BeginLayer]);
            assert_eq!(gesture.phase(), Phase::Held);
            assert_eq!(
                gesture.handle(Event::Release),
                vec![Effect::Invalidate, Effect::EndLayer, Effect::Consume]
            );
        }
    }

    #[test]
    fn test_phonetic_priority_excludes_english_and_modifiers() {
        // 英文模式下不觸發注音優先
        assert!(!prefers_phonetic_f_chord(0x55, true, false, true, false));
        assert!(!prefers_phonetic_f_chord(0x4D, true, false, true, false));

        // 帶有修飾鍵 (Shift/Ctrl/Alt) 不觸發注音優先
        assert!(!prefers_phonetic_f_chord(0x55, true, true, true, true));
        assert!(!prefers_phonetic_f_chord(0x4D, true, true, true, true));

        // 非標準注音配置 (例如許氏) 不觸發注音優先
        assert!(!prefers_phonetic_f_chord(0x55, true, true, false, false));
        assert!(!prefers_phonetic_f_chord(0x4D, true, true, false, false));
    }
}
