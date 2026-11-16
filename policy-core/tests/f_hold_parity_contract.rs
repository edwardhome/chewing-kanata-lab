// SPDX-License-Identifier: MIT
//! Regression and contract tests for FHoldGesture alignment with Mac c2fe36e semantics.

use std::time::Duration;
use chewing_policy_core::f_hold::{
    can_start_chord, is_phonetic_f_overlap, prefers_phonetic_f_chord,
    Effect as FHoldEffect, Event as FHoldEvent, FHoldGesture, Phase as FHoldPhase,
};
use chewing_policy_core::shift_letter::{FCompositionBuffer, ShiftLetterSession};
use chewing_policy_core::TechnicalLexicon;

#[test]
fn test_delay_clamping_and_snapshot_defaults() {
    // 預設延遲為 150ms
    let default_gesture = FHoldGesture::default();
    assert_eq!(default_gesture.delay, Duration::from_millis(FHoldGesture::DEFAULT_DELAY_MS));
    assert_eq!(default_gesture.phase(), FHoldPhase::Idle);
    assert!(!default_gesture.owns_key_up());

    // 邊界夾取：下限 100ms，上限 1000ms
    let too_short = FHoldGesture::new(Duration::from_millis(50));
    assert_eq!(too_short.delay, Duration::from_millis(FHoldGesture::MIN_DELAY_MS));

    let too_long = FHoldGesture::new(Duration::from_millis(2000));
    assert_eq!(too_long.delay, Duration::from_millis(FHoldGesture::MAX_DELAY_MS));

    let normal = FHoldGesture::new(Duration::from_millis(300));
    assert_eq!(normal.delay, Duration::from_millis(300));
}

#[test]
fn test_short_tap_lifecycle_and_replays_tap() {
    let delay = Duration::from_millis(150);
    let mut gesture = FHoldGesture::new(delay);

    // 1. 初次 Press (eligible, not repeating)
    let effects = gesture.handle(FHoldEvent::Press { eligible: true, repeating: false });
    assert_eq!(effects, vec![FHoldEffect::Schedule, FHoldEffect::Consume]);
    assert_eq!(gesture.phase(), FHoldPhase::Pending);
    assert!(gesture.owns_key_up());

    // 2. 在延遲之前任意時間放開 (例如 50ms) -> 判定為短按點擊
    gesture.handle(FHoldEvent::Elapsed(Duration::from_millis(50)));
    let release_effects = gesture.handle(FHoldEvent::Release);
    assert_eq!(
        release_effects,
        vec![FHoldEffect::Invalidate, FHoldEffect::ReplayTap, FHoldEffect::Consume]
    );
    assert_eq!(gesture.phase(), FHoldPhase::Idle);
    assert!(!gesture.owns_key_up());

    // 3. 額外的 Release 應為空效果
    assert!(gesture.handle(FHoldEvent::Release).is_empty());
}

#[test]
fn test_strict_boundary_elapsed_vs_delay() {
    for &ms in &[100u64, 150u64, 200u64, 500u64, 1000u64] {
        let delay = Duration::from_millis(ms);

        // Case A: elapsed < delay
        let mut g_less = FHoldGesture::new(delay);
        g_less.handle(FHoldEvent::Press { eligible: true, repeating: false });
        let eff_less = g_less.handle(FHoldEvent::Elapsed(delay - Duration::from_millis(1)));
        assert!(eff_less.is_empty(), "elapsed < delay must not trigger timeout");
        assert_eq!(g_less.phase(), FHoldPhase::Pending);

        // Case B: elapsed == delay (Strict boundary requirement: must NOT trigger layer)
        let mut g_equal = FHoldGesture::new(delay);
        g_equal.handle(FHoldEvent::Press { eligible: true, repeating: false });
        let eff_equal = g_equal.handle(FHoldEvent::Elapsed(delay));
        assert!(eff_equal.is_empty(), "elapsed == delay must strictly not enter layer");
        assert_eq!(g_equal.phase(), FHoldPhase::Pending);

        // Case C: elapsed > delay (Strictly greater than -> triggers BeginLayer)
        let mut g_greater = FHoldGesture::new(delay);
        g_greater.handle(FHoldEvent::Press { eligible: true, repeating: false });
        let eff_greater = g_greater.handle(FHoldEvent::Elapsed(delay + Duration::from_millis(1)));
        assert_eq!(eff_greater, vec![FHoldEffect::BeginLayer]);
        assert_eq!(g_greater.phase(), FHoldPhase::Held);
        assert!(g_greater.owns_key_up());

        // Release from Held phase ends layer
        let eff_rel = g_greater.handle(FHoldEvent::Release);
        assert_eq!(
            eff_rel,
            vec![FHoldEffect::Invalidate, FHoldEffect::EndLayer, FHoldEffect::Consume]
        );
        assert_eq!(g_greater.phase(), FHoldPhase::Idle);
        assert!(!g_greater.owns_key_up());
    }
}

#[test]
fn test_other_key_before_or_at_deadline_cancels_layer_for_r_u_m_escape() {
    let delay = Duration::from_millis(150);

    // 測試各種不同時間點下的 OtherKey (包括 0ms, 75ms, 150ms 剛好在 deadline 上)
    for elapsed_ms in [0, 50, 100, 150] {
        let mut gesture = FHoldGesture::new(delay);
        gesture.handle(FHoldEvent::Press { eligible: true, repeating: false });

        if elapsed_ms > 0 {
            let eff = gesture.handle(FHoldEvent::Elapsed(Duration::from_millis(elapsed_ms)));
            assert!(eff.is_empty());
        }

        // 當按下任何其他鍵（如 R, U, M, Esc）
        let effects = gesture.handle(FHoldEvent::OtherKey);
        // 必須判定為普通打字 (ReplayTap)，收回計時器 (Invalidate)，且絕不提前進入符號層
        assert_eq!(
            effects,
            vec![FHoldEffect::Invalidate, FHoldEffect::ReplayTap],
            "OtherKey at {}ms must cancel gesture and replay original tap",
            elapsed_ms
        );
        assert_eq!(gesture.phase(), FHoldPhase::Idle);
        // owns_key_up 仍保留，確保後續 F 實體 release 仍能被正確 consume 避免洩漏至外部
        assert!(gesture.owns_key_up());
    }
}

#[test]
fn test_repeats_and_late_timeouts_cannot_reenter_after_cancellation() {
    let delay = Duration::from_millis(150);
    let mut gesture = FHoldGesture::new(delay);
    gesture.handle(FHoldEvent::Press { eligible: true, repeating: false });

    // 觸發 OtherKey 轉為一般打字
    let effects = gesture.handle(FHoldEvent::OtherKey);
    assert_eq!(effects, vec![FHoldEffect::Invalidate, FHoldEffect::ReplayTap]);
    assert_eq!(gesture.phase(), FHoldPhase::Idle);
    assert!(gesture.owns_key_up());

    // 1. 晚到的計時器 Elapsed (> 150ms) 絕不能重新啟用
    let eff_late_elapsed = gesture.handle(FHoldEvent::Elapsed(Duration::from_millis(300)));
    assert!(eff_late_elapsed.is_empty());
    assert_eq!(gesture.phase(), FHoldPhase::Idle);

    // 2. 晚到的 Timeout 事件絕不能重新啟用
    let eff_late_timeout = gesture.handle(FHoldEvent::Timeout);
    assert!(eff_late_timeout.is_empty());
    assert_eq!(gesture.phase(), FHoldPhase::Idle);

    // 3. F 鍵自動重複 (repeating: true) 必須被 Consume 且不可重新進層或進入 Pending
    let eff_repeat = gesture.handle(FHoldEvent::Press { eligible: true, repeating: true });
    assert_eq!(eff_repeat, vec![FHoldEffect::Consume]);
    assert_eq!(gesture.phase(), FHoldPhase::Idle);

    // 4. 重複的 OtherKey 在 Idle 狀態下不做事
    assert!(gesture.handle(FHoldEvent::OtherKey).is_empty());

    // 5. 最終實體 F 放開 (Release)，必須 consume 並釋放 key_up ownership
    let eff_release = gesture.handle(FHoldEvent::Release);
    assert_eq!(eff_release, vec![FHoldEffect::Consume]);
    assert_eq!(gesture.phase(), FHoldPhase::Idle);
    assert!(!gesture.owns_key_up());

    // 釋放之後的重複 Release 應為空
    assert!(gesture.handle(FHoldEvent::Release).is_empty());
}

#[test]
fn test_observed_release_and_delayed_real_release() {
    let delay = Duration::from_millis(150);
    let mut gesture = FHoldGesture::new(delay);
    gesture.handle(FHoldEvent::Press { eligible: true, repeating: false });

    // 被動放開事件 (ObservedRelease，如焦點切換或外部監聽截獲)
    let eff_obs = gesture.handle(FHoldEvent::ObservedRelease);
    assert_eq!(
        eff_obs,
        vec![FHoldEffect::Invalidate, FHoldEffect::ReplayTap, FHoldEffect::Consume]
    );
    assert_eq!(gesture.phase(), FHoldPhase::Idle);
    // 關鍵語意：ObservedRelease 發生後，gesture 仍維持 owns_key_up = true，
    // 以便當真實的實體 Release 抵達時將其 consume，防止二次處理
    assert!(gesture.owns_key_up());

    // 延遲到達的真實 Release
    let eff_real = gesture.handle(FHoldEvent::Release);
    assert_eq!(eff_real, vec![FHoldEffect::Consume]);
    assert!(!gesture.owns_key_up());
}

#[test]
fn test_observed_release_while_held() {
    let delay = Duration::from_millis(150);
    let mut gesture = FHoldGesture::new(delay);
    gesture.handle(FHoldEvent::Press { eligible: true, repeating: false });
    gesture.handle(FHoldEvent::Elapsed(Duration::from_millis(160)));
    assert_eq!(gesture.phase(), FHoldPhase::Held);

    // 在 Held 狀態下觸發 ObservedRelease
    let eff_obs = gesture.handle(FHoldEvent::ObservedRelease);
    assert_eq!(
        eff_obs,
        vec![FHoldEffect::Invalidate, FHoldEffect::EndLayer, FHoldEffect::Consume]
    );
    assert_eq!(gesture.phase(), FHoldPhase::Idle);
    assert!(gesture.owns_key_up());

    // 後續真實 Release
    assert_eq!(gesture.handle(FHoldEvent::Release), vec![FHoldEffect::Consume]);
    assert!(!gesture.owns_key_up());
}

#[test]
fn test_cancel_event_resets_ownership_and_layer() {
    // Pending 狀態下 Cancel
    let mut g1 = FHoldGesture::new(Duration::from_millis(150));
    g1.handle(FHoldEvent::Press { eligible: true, repeating: false });
    assert!(g1.owns_key_up());
    let eff_cancel_pending = g1.handle(FHoldEvent::Cancel);
    assert_eq!(eff_cancel_pending, vec![FHoldEffect::Invalidate]);
    assert_eq!(g1.phase(), FHoldPhase::Idle);
    assert!(!g1.owns_key_up());

    // Held 狀態下 Cancel
    let mut g2 = FHoldGesture::new(Duration::from_millis(150));
    g2.handle(FHoldEvent::Press { eligible: true, repeating: false });
    g2.handle(FHoldEvent::Elapsed(Duration::from_millis(160)));
    assert_eq!(g2.phase(), FHoldPhase::Held);
    assert!(g2.owns_key_up());
    let eff_cancel_held = g2.handle(FHoldEvent::Cancel);
    assert_eq!(eff_cancel_held, vec![FHoldEffect::Invalidate, FHoldEffect::EndLayer]);
    assert_eq!(g2.phase(), FHoldPhase::Idle);
    assert!(!g2.owns_key_up());
}

#[test]
fn test_ineligible_press_does_not_start_gesture() {
    let mut gesture = FHoldGesture::new(Duration::from_millis(150));
    // eligible: false (例如有不支援的修飾鍵或在密碼欄中)
    let effects = gesture.handle(FHoldEvent::Press { eligible: false, repeating: false });
    assert!(effects.is_empty());
    assert_eq!(gesture.phase(), FHoldPhase::Idle);
    assert!(!gesture.owns_key_up());
}

#[test]
fn test_shift_letter_and_f_layer_buffer_interaction() {
    let lexicon = TechnicalLexicon::default();

    // 驗證 Shift+F 不會啟動 InitialLetterHold（Shift+F 保留給英文首字或大寫，非長按分層）
    let shift_f_session = ShiftLetterSession::start_with_letter('F', &lexicon);
    assert_eq!(shift_f_session.initial_letter_key(), None);

    // 驗證 FCompositionBuffer 在分層輸入時保護預先存在的英文字詞
    let base_session = ShiftLetterSession::start_with_letter('S', &lexicon);
    let mut buffer = FCompositionBuffer::new(Some(base_session), &lexicon);

    // 在 F 符號層輸入符號
    buffer.append("!");
    buffer.append("@");
    assert_eq!(buffer.symbols(), "!@");
    assert_eq!(buffer.preview(&lexicon).raw, "S!@");

    // 退格只刪除符號層的符號，絕不破壞原先的 'S'
    buffer.backspace();
    assert_eq!(buffer.symbols(), "!");
    assert_eq!(buffer.preview(&lexicon).raw, "S!");

    buffer.backspace();
    assert_eq!(buffer.symbols(), "");
    assert_eq!(buffer.preview(&lexicon).raw, "S");

    // 即使多退幾次，也不會刪除 base
    buffer.backspace();
    assert_eq!(buffer.symbols(), "");
    assert_eq!(buffer.preview(&lexicon).raw, "S");
}

#[test]
fn test_phonetic_helpers_and_chord_guards() {
    // 0x55 (U / ㄧ) 與 0x4D (M / ㄩ) 為標準注音 F (ㄑ) 重疊音節
    assert!(is_phonetic_f_overlap(0x55));
    assert!(is_phonetic_f_overlap(0x4D));
    assert!(!is_phonetic_f_overlap(0x52)); // R 不是重疊音節

    // prefers_phonetic_f_chord 在注音、標準鍵盤、無修飾鍵且 pending 時成立
    assert!(prefers_phonetic_f_chord(0x55, true, true, true, false));
    assert!(!prefers_phonetic_f_chord(0x55, false, true, true, false)); // not pending
    assert!(!prefers_phonetic_f_chord(0x55, true, false, true, false)); // not chinese
    assert!(!prefers_phonetic_f_chord(0x55, true, true, false, false)); // not default layout
    assert!(!prefers_phonetic_f_chord(0x55, true, true, true, true));  // has modifiers

    // can_start_chord: repeating 或 prefers_phonetic 均禁止立即 chord
    assert!(!can_start_chord(true, true, false, false));  // repeating
    assert!(!can_start_chord(true, false, true, false));  // disallowed modifiers
    assert!(!can_start_chord(true, false, false, true));  // prefers phonetic
    assert!(!can_start_chord(false, false, false, false)); // no output
    assert!(can_start_chord(true, false, false, false));  // valid chord condition
}
