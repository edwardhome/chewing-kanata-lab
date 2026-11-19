// SPDX-License-Identifier: MIT
//! F 鍵長按與分層手勢判定器 (FHoldGesture)
//! 依據 Mac c2fe36e (QuickSymbolLayer.swift 中的 FHoldGesture 與 quick-symbols.md) 實作手勢語意對齊：
//!
//! 1. 按下 F 鍵立即顯示尚未提交的可撤回原生預覽字元（標準注音為「ㄑ」，英文/ASCII 為 `f`）。
//! 2. 門檻內（含等於門檻）放開 F，或按任何下一鍵（包括 R/U/M/Esc）：取消長按手勢，送回原 F 一次，
//!    再按正常流程處理下一鍵（移除立即 chord 行為，不提前進層）。
//! 3. 只有經過時間嚴格大於設定延遲（elapsed > delay）且手勢仍在等待中（pending）才觸發逾時，
//!    撤回 F 的預覽並進入符號層（Held 狀態）。
//! 4. 一旦轉入一般打字（非 F 層），晚到的計時器逾時或按住 F 的重複事件絕不重新啟用分層，直到實體 F 放開。
//! 5. 支援被動放開監聽（ObservedRelease）與標準放開（Release）。

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
    OtherKey,
    LayerCommand,
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
    layer_used: bool,
}

impl Default for FHoldGesture {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(Self::DEFAULT_DELAY_MS),
            phase: Phase::Idle,
            owns_key_up: false,
            layer_used: false,
        }
    }
}

impl FHoldGesture {
    pub const DEFAULT_DELAY_MS: u64 = 150;
    pub const MIN_DELAY_MS: u64 = 100;
    pub const MAX_DELAY_MS: u64 = 1000;

    pub fn clamped_delay_ms(ms: u64) -> u64 {
        ms.clamp(Self::MIN_DELAY_MS, Self::MAX_DELAY_MS)
    }

    pub fn new(delay: Duration) -> Self {
        let clamped_ms = Self::clamped_delay_ms(delay.as_millis() as u64);
        Self {
            delay: Duration::from_millis(clamped_ms),
            phase: Phase::Idle,
            owns_key_up: false,
            layer_used: false,
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
                self.layer_used = false;
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
                    Phase::Held => {
                        let mut effects = vec![Effect::Invalidate, Effect::EndLayer];
                        if !self.layer_used { effects.push(Effect::ReplayTap); }
                        effects.push(Effect::Consume);
                        effects
                    },
                    Phase::Idle => {
                        if owned {
                            vec![Effect::Consume]
                        } else {
                            vec![]
                        }
                    }
                }
            }
            Event::LayerCommand => {
                if self.phase == Phase::Held { self.layer_used = true; }
                vec![]
            }
            Event::OtherKey => {
                if self.phase != Phase::Pending {
                    return vec![];
                }
                // 門檻內（含等於門檻）任何下一鍵都屬正常打字；只能由 elapsed 判斷長按。
                self.phase = Phase::Idle;
                vec![Effect::Invalidate, Effect::ReplayTap]
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
/// 保留此輔助函式供相容與測試使用
pub fn is_phonetic_f_overlap(vk: u16) -> bool {
    // 0x55: VK_U ('u' / ㄧ)
    // 0x4D: VK_M ('m' / ㄩ)
    vk == 0x55 || vk == 0x4D
}

/// 判斷在目前狀態下是否應優先保留注音 (ㄑㄧ / ㄑㄩ) 而非觸發 F 立即組合鍵 (8 / 2)
/// 在新語意下，長按門檻內任何鍵均不進層，因此 pending 狀態下一律優先保留普通打字
pub fn prefers_phonetic_f_chord(
    vk: u16,
    is_pending: bool,
    is_chinese_mode: bool,
    is_default_layout: bool,
    has_modifiers: bool,
) -> bool {
    is_pending && is_chinese_mode && is_default_layout && !has_modifiers && is_phonetic_f_overlap(vk)
}

/// 判斷是否能啟動 F 組合鍵
/// 在新語意下，未超逾門檻前不得立即 chord
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
    fn test_exact_deadline_boundary_and_strict_greater_than() {
        for &ms in &[100u64, 150u64, 250u64, 1000u64] {
            let delay = Duration::from_millis(ms);
            let mut gesture = FHoldGesture::new(delay);
            assert_eq!(
                gesture.handle(Event::Press { eligible: true, repeating: false }),
                vec![Effect::Schedule, Effect::Consume]
            );
            assert_eq!(gesture.phase(), Phase::Pending);

            // 門檻之前 (delay - 1ms) -> 不觸發
            assert!(gesture.handle(Event::Elapsed(delay - Duration::from_millis(1))).is_empty());
            assert_eq!(gesture.phase(), Phase::Pending);

            // 剛好等於門檻 (delay) -> 依然不觸發（必須 strictly greater than）
            assert!(gesture.handle(Event::Elapsed(delay)).is_empty());
            assert_eq!(gesture.phase(), Phase::Pending);

            // 嚴格大於門檻 (delay + 1ms) -> 觸發 Timeout 並進入 Held
            assert_eq!(
                gesture.handle(Event::Elapsed(delay + Duration::from_millis(1))),
                vec![Effect::BeginLayer]
            );
            assert_eq!(gesture.phase(), Phase::Held);

            // 放開 F -> Invalidate, EndLayer, Consume
            assert_eq!(
                gesture.handle(Event::Release),
                vec![Effect::Invalidate, Effect::EndLayer, Effect::ReplayTap, Effect::Consume]
            );
            assert_eq!(gesture.phase(), Phase::Idle);
        }
    }

    #[test]
    fn test_any_other_key_at_or_before_deadline_resolves_ordinary_typing() {
        // 包含 R, U, M, Esc 在門檻內按下，均應立即判定為普通打字 (ReplayTap)，絕不提前進層
        for &ms in &[100u64, 150u64, 250u64] {
            let delay = Duration::from_millis(ms);

            // 在 0ms (立即按下一鍵)
            let mut g1 = FHoldGesture::new(delay);
            g1.handle(Event::Press { eligible: true, repeating: false });
            assert_eq!(
                g1.handle(Event::OtherKey),
                vec![Effect::Invalidate, Effect::ReplayTap]
            );
            assert_eq!(g1.phase(), Phase::Idle);

            // 在門檻前半段
            let mut g2 = FHoldGesture::new(delay);
            g2.handle(Event::Press { eligible: true, repeating: false });
            g2.handle(Event::Elapsed(delay / 2));
            assert_eq!(
                g2.handle(Event::OtherKey),
                vec![Effect::Invalidate, Effect::ReplayTap]
            );
            assert_eq!(g2.phase(), Phase::Idle);

            // 在剛好等於門檻時
            let mut g3 = FHoldGesture::new(delay);
            g3.handle(Event::Press { eligible: true, repeating: false });
            g3.handle(Event::Elapsed(delay));
            assert_eq!(
                g3.handle(Event::OtherKey),
                vec![Effect::Invalidate, Effect::ReplayTap]
            );
            assert_eq!(g3.phase(), Phase::Idle);
        }
    }

    #[test]
    fn test_late_timeout_and_repeats_cannot_reactivate_after_ordinary_typing() {
        let delay = Duration::from_millis(150);
        let mut gesture = FHoldGesture::new(delay);
        gesture.handle(Event::Press { eligible: true, repeating: false });

        // 遭遇 OtherKey (例如 R, U, M, Esc) 解決為普通打字
        let effects = gesture.handle(Event::OtherKey);
        assert_eq!(effects, vec![Effect::Invalidate, Effect::ReplayTap]);
        assert_eq!(gesture.phase(), Phase::Idle);

        // 雖然實體 F 鍵尚未放開 (owns_key_up 仍為 true)，但 phase 已是 Idle
        // 1. 晚到的計時器 Elapsed (> 150ms) 絕不能重新啟用
        assert!(gesture.handle(Event::Elapsed(Duration::from_millis(200))).is_empty());
        assert_eq!(gesture.phase(), Phase::Idle);

        // 2. 晚到的 Timeout 絕不能重新啟用
        assert!(gesture.handle(Event::Timeout).is_empty());
        assert_eq!(gesture.phase(), Phase::Idle);

        // 3. F 鍵自動重複 (repeating: true) 必須被 consume 且絕不能重新進入 Pending 或 Held
        assert_eq!(
            gesture.handle(Event::Press { eligible: true, repeating: true }),
            vec![Effect::Consume]
        );
        assert_eq!(gesture.phase(), Phase::Idle);

        // 4. 其他鍵也不會做任何事
        assert!(gesture.handle(Event::OtherKey).is_empty());

        // 5. 直到真正收到實體 F 放開 (Release 或 ObservedRelease)
        assert_eq!(gesture.handle(Event::Release), vec![Effect::Consume]);
        assert_eq!(gesture.phase(), Phase::Idle);
        assert!(!gesture.owns_key_up());

        // 後續重複的 Release 什麼都不做
        assert!(gesture.handle(Event::Release).is_empty());
    }

    #[test]
    fn test_short_tap_f_release_replays_tap() {
        let delay = Duration::from_millis(150);
        let mut gesture = FHoldGesture::new(delay);
        gesture.handle(Event::Press { eligible: true, repeating: false });

        // 在 50ms 放開 F
        gesture.handle(Event::Elapsed(Duration::from_millis(50)));
        assert_eq!(
            gesture.handle(Event::Release),
            vec![Effect::Invalidate, Effect::ReplayTap, Effect::Consume]
        );
        assert_eq!(gesture.phase(), Phase::Idle);
    }

    #[test]
    fn test_observed_release_maintains_ownership_until_genuine_release() {
        let delay = Duration::from_millis(150);
        let mut gesture = FHoldGesture::new(delay);
        gesture.handle(Event::Press { eligible: true, repeating: false });

        // 被動監聽先觀察到放開
        let effects = gesture.handle(Event::ObservedRelease);
        assert_eq!(effects, vec![Effect::Invalidate, Effect::ReplayTap, Effect::Consume]);
        assert_eq!(gesture.phase(), Phase::Idle);
        assert!(gesture.owns_key_up()); // 仍然保有 ownership 直到真實 release 事件

        // 真實 release 事件抵達
        assert_eq!(gesture.handle(Event::Release), vec![Effect::Consume]);
        assert!(!gesture.owns_key_up());
    }

    #[test]
    fn test_cancel_handles_pending_and_held() {
        let delay = Duration::from_millis(150);
        let mut g1 = FHoldGesture::new(delay);
        g1.handle(Event::Press { eligible: true, repeating: false });
        assert_eq!(g1.handle(Event::Cancel), vec![Effect::Invalidate]);
        assert_eq!(g1.phase(), Phase::Idle);

        let mut g2 = FHoldGesture::new(delay);
        g2.handle(Event::Press { eligible: true, repeating: false });
        g2.handle(Event::Elapsed(Duration::from_millis(151)));
        assert_eq!(g2.phase(), Phase::Held);
        assert_eq!(g2.handle(Event::Cancel), vec![Effect::Invalidate, Effect::EndLayer]);
        assert_eq!(g2.phase(), Phase::Idle);
    }
}
