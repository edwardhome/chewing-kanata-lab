// SPDX-License-Identifier: MIT
//! Deterministic scheduling for uncommitted-composition prediction.
//!
//! This module performs no I/O and never logs composition text. The adapter must
//! cancel its worker on invalidation, enforce the network deadline, validate the
//! response with `prediction::validate_and_extract_continuation`, and keep that
//! response paired with its request token. A timed-out worker still owns the
//! in-flight slot until it reports completion: a new thread must not replace it.

pub const DEBOUNCE_MS: u64 = 700;
pub const MIN_REQUEST_INTERVAL_MS: u64 = 2_000;
pub const REQUEST_DEADLINE_MS: u64 = 2_000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    Safe,
    Secure,
    Unknown,
}

/// All identity values are adapter-issued, not inferred from composition text.
/// Change `mode_id` on language, model, or prediction-setting changes.
/// Selection offsets use Unicode scalar positions. No Debug implementation:
/// formatting a context must not accidentally expose typed text to a log.
#[derive(Clone, PartialEq, Eq)]
pub struct PredictionContext {
    pub prefix: String,
    pub focus_id: u64,
    pub document_id: u64,
    pub mode_id: u64,
    pub selection_start: usize,
    pub selection_len: usize,
    pub complete: bool,
    pub enabled: bool,
    pub safety: Safety,
}

impl PredictionContext {
    fn eligible(&self) -> bool {
        let length = self.prefix.chars().count();
        self.enabled
            && self.safety == Safety::Safe
            && self.complete
            && (1..=256).contains(&length)
            && self.selection_len == 0
            && self.selection_start == length
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestToken(u64);

pub struct PredictionRequest {
    pub token: RequestToken,
    pub context: PredictionContext,
}

/// One instance per input-method service, not a machine-wide inference lock.
/// Supply monotonic milliseconds from the same clock for every call.
#[derive(Default)]
pub struct PredictionSession {
    generation: u64,
    exhausted: bool,
    current: Option<PredictionContext>,
    changed_at: u64,
    last_started_at: Option<u64>,
    dispatched_generation: Option<u64>,
    in_flight: Option<(RequestToken, u64)>,
    ready: Option<RequestToken>,
}

impl PredictionSession {
    /// Keep only the latest snapshot; no request queue is allocated.
    /// Returns whether the previous suggestion/request has been invalidated.
    pub fn observe(&mut self, context: Option<PredictionContext>, now_ms: u64) -> bool {
        if self.current == context {
            return false;
        }
        self.advance_generation();
        self.current = if self.exhausted { None } else { context };
        self.changed_at = now_ms;
        true
    }

    /// Use for Esc, F gestures, composition cancellation and equivalent events,
    /// even when the visible text happens to remain unchanged.
    pub fn invalidate(&mut self) {
        self.advance_generation();
        self.current = None;
    }

    fn advance_generation(&mut self) {
        self.ready = None;
        match self.generation.checked_add(1) {
            Some(next) => self.generation = next,
            None => self.exhausted = true,
        }
    }

    pub fn poll(&mut self, now_ms: u64) -> Option<PredictionRequest> {
        if self.exhausted
            || self.in_flight.is_some()
            || self.dispatched_generation == Some(self.generation)
        {
            return None;
        }
        let context = self.current.as_ref()?;
        if !context.eligible() || now_ms.checked_sub(self.changed_at)? < DEBOUNCE_MS {
            return None;
        }
        if let Some(last) = self.last_started_at {
            if now_ms.checked_sub(last)? < MIN_REQUEST_INTERVAL_MS {
                return None;
            }
        }
        let request = PredictionRequest {
            token: RequestToken(self.generation),
            context: context.clone(),
        };
        self.last_started_at = Some(now_ms);
        self.dispatched_generation = Some(self.generation);
        self.in_flight = Some((request.token, now_ms));
        Some(request)
    }

    /// Call only once the worker has actually stopped. `valid_prediction` means
    /// HTTP/JSON and exact-prefix validation all passed. Failures never retry the
    /// same unchanged snapshot and never authorize acceptance.
    pub fn complete(
        &mut self,
        token: RequestToken,
        now_ms: u64,
        valid_prediction: bool,
    ) -> bool {
        let Some((active, started)) = self.in_flight else { return false };
        if active != token {
            return false;
        }
        self.in_flight = None;
        let timely = now_ms.checked_sub(started).is_some_and(|ms| ms < REQUEST_DEADLINE_MS);
        let current = !self.exhausted
            && token.0 == self.generation
            && self.current.as_ref().is_some_and(PredictionContext::eligible);
        if valid_prediction && timely && current {
            self.ready = Some(token);
            true
        } else {
            self.ready = None;
            false
        }
    }

    /// Call after explicit user selection, immediately before appending the
    /// validated suffix. Recheck the live context even if no observe event ran.
    /// Acceptance is single-use. This function never commits or edits text.
    pub fn accept(&mut self, token: RequestToken, live: &PredictionContext) -> bool {
        if self.current.as_ref() != Some(live) || !live.eligible() {
            self.invalidate();
            return false;
        }
        if self.ready != Some(token) || token.0 != self.generation || self.exhausted {
            return false;
        }
        self.ready = None;
        true
    }

    pub fn has_in_flight_request(&self) -> bool {
        self.in_flight.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(text: &str) -> PredictionContext {
        PredictionContext {
            prefix: text.into(), focus_id: 1, document_id: 1, mode_id: 1,
            selection_start: text.chars().count(), selection_len: 0,
            complete: true, enabled: true, safety: Safety::Safe,
        }
    }

    fn started() -> (PredictionSession, PredictionRequest) {
        let mut session = PredictionSession::default();
        session.observe(Some(context("今天晚上")), 0);
        let request = session.poll(700).unwrap();
        (session, request)
    }

    #[test]
    fn debounce_uses_latest_edit_and_does_not_restart_for_identical_observation() {
        let mut s = PredictionSession::default();
        s.observe(Some(context("a")), 0);
        assert!(!s.observe(Some(context("a")), 600));
        assert!(s.poll(699).is_none());
        s.observe(Some(context("ab")), 699);
        assert!(s.poll(1398).is_none());
        assert_eq!(s.poll(1399).unwrap().context.prefix, "ab");
    }

    #[test]
    fn unsafe_incomplete_selected_and_nonterminal_contexts_never_start() {
        let base = context("中文");
        let mut cases = vec![context(""), context(&"字".repeat(257))];
        let mut c = base.clone(); c.safety = Safety::Secure; cases.push(c);
        let mut c = base.clone(); c.safety = Safety::Unknown; cases.push(c);
        let mut c = base.clone(); c.enabled = false; cases.push(c);
        let mut c = base.clone(); c.complete = false; cases.push(c);
        let mut c = base.clone(); c.selection_start = 1; cases.push(c);
        let mut c = base; c.selection_len = 1; cases.push(c);
        for c in cases {
            let mut s = PredictionSession::default();
            s.observe(Some(c), 0);
            assert!(s.poll(700).is_none());
        }
    }

    #[test]
    fn accepts_256_scalars_and_counts_non_bmp_as_one() {
        let mut s = PredictionSession::default();
        s.observe(Some(context(&"𠮷".repeat(256))), 0);
        assert!(s.poll(700).is_some());
    }

    #[test]
    fn only_latest_pending_context_runs_after_worker_stops_and_rate_limit_passes() {
        let (mut s, first) = started();
        s.observe(Some(context("B")), 800);
        s.observe(Some(context("C")), 900);
        assert!(s.poll(5000).is_none());
        assert!(s.has_in_flight_request());
        assert!(!s.complete(first.token, 2000, true));
        assert!(s.poll(2699).is_none());
        assert_eq!(s.poll(2700).unwrap().context.prefix, "C");
    }

    #[test]
    fn identical_text_after_edit_is_a_different_generation() {
        let (mut s, first) = started();
        s.observe(Some(context("B")), 800);
        s.observe(Some(first.context.clone()), 900);
        assert!(!s.complete(first.token, 1000, true));
        assert_ne!(s.poll(2700).unwrap().token, first.token);
    }

    #[test]
    fn cancelled_worker_holds_slot_until_completion() {
        let (mut s, first) = started();
        s.invalidate();
        s.observe(Some(context("new")), 900);
        assert!(s.poll(10000).is_none());
        assert!(!s.complete(first.token, 10000, true));
        assert!(s.poll(10000).is_some());
    }

    #[test]
    fn total_deadline_rejects_at_boundary() {
        let (mut s, request) = started();
        assert!(!s.complete(request.token, 2700, true));
        assert!(!s.accept(request.token, &request.context));
    }

    #[test]
    fn acceptance_requires_valid_completion_and_is_single_use() {
        let (mut s, r) = started();
        assert!(!s.accept(r.token, &r.context));
        assert!(s.complete(r.token, 2699, true));
        assert!(s.accept(r.token, &r.context));
        assert!(!s.accept(r.token, &r.context));
        assert!(s.poll(10000).is_none());
    }

    #[test]
    fn failed_prediction_does_not_retry_unchanged_text() {
        let (mut s, r) = started();
        assert!(!s.complete(r.token, 800, false));
        assert!(!s.accept(r.token, &r.context));
        assert!(s.poll(10000).is_none());
    }

    #[test]
    fn focus_document_mode_and_safety_changes_reject_late_results() {
        for change in 0..5 {
            let (mut s, r) = started();
            let mut c = r.context.clone();
            match change {
                0 => c.focus_id += 1,
                1 => c.document_id += 1,
                2 => c.mode_id += 1,
                3 => c.safety = Safety::Unknown,
                _ => c.enabled = false,
            }
            s.observe(Some(c), 800);
            assert!(!s.complete(r.token, 900, true));
        }
    }

    #[test]
    fn acceptance_rechecks_live_context_without_relying_on_observe() {
        let (mut s, r) = started();
        assert!(s.complete(r.token, 800, true));
        let mut changed = r.context.clone(); changed.document_id += 1;
        assert!(!s.accept(r.token, &changed));
        assert!(!s.accept(r.token, &r.context));
    }

    #[test]
    fn duplicate_old_completion_does_not_release_new_worker_slot() {
        let (mut s, old) = started();
        assert!(s.complete(old.token, 800, true));
        s.observe(Some(context("new")), 900);
        let new = s.poll(2700).unwrap();
        assert!(!s.complete(old.token, 2800, true));
        assert!(s.has_in_flight_request());
        assert!(s.complete(new.token, 2800, true));
    }

    #[test]
    fn invalidation_discards_ready_result() {
        let (mut s, r) = started();
        assert!(s.complete(r.token, 800, true));
        s.invalidate();
        assert!(!s.accept(r.token, &r.context));
    }

    #[test]
    fn backward_clock_cannot_start_or_accept_result() {
        let mut s = PredictionSession::default();
        s.observe(Some(context("a")), 500);
        assert!(s.poll(400).is_none());
        let r = s.poll(1200).unwrap();
        assert!(!s.complete(r.token, 1199, true));
    }

    #[test]
    fn generation_overflow_fails_closed() {
        let mut s = PredictionSession { generation: u64::MAX, ..Default::default() };
        s.observe(Some(context("a")), 0);
        assert!(s.poll(10000).is_none());
        s.observe(Some(context("b")), 11000);
        assert!(s.poll(12000).is_none());
    }
}
