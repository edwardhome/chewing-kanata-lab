use chewing_policy_core::prediction::{validate_and_extract_continuation, PredictionLanguage};
use chewing_policy_core::prediction_session::{PredictionContext, PredictionSession, Safety};

fn context(prefix: &str) -> PredictionContext {
    PredictionContext {
        prefix: prefix.into(),
        focus_id: 1,
        document_id: 1,
        mode_id: 1,
        selection_start: prefix.chars().count(),
        selection_len: 0,
        complete: true,
        enabled: true,
        safety: Safety::Safe,
    }
}

#[test]
fn valid_result_preserves_original_until_explicit_acceptance() {
    for (prefix, candidate, language) in [
        ("今天晚上", "今天晚上一起吃飯", PredictionLanguage::TraditionalChinese),
        ("machine", "machine learning", PredictionLanguage::English),
    ] {
        let live = context(prefix);
        let mut session = PredictionSession::default();
        session.observe(Some(live.clone()), 0);
        let request = session.poll(700).unwrap();
        let suffix = validate_and_extract_continuation(&request.context.prefix, candidate, language);
        assert!(session.complete(request.token, 800, suffix.is_some()));
        assert_eq!(live.prefix, prefix);
        assert!(session.accept(request.token, &live));
        assert_eq!(format!("{}{}", live.prefix, suffix.unwrap()), candidate);
        assert!(!session.accept(request.token, &live));
    }
}

#[test]
fn rewritten_prefix_never_becomes_an_acceptable_prediction() {
    let live = context("USB");
    let mut session = PredictionSession::default();
    session.observe(Some(live.clone()), 0);
    let request = session.poll(700).unwrap();
    let suffix = validate_and_extract_continuation(
        &request.context.prefix, "usb connection", PredictionLanguage::English,
    );
    assert!(suffix.is_none());
    assert!(!session.complete(request.token, 800, suffix.is_some()));
    assert!(!session.accept(request.token, &live));
    assert_eq!(live.prefix, "USB");
}

#[test]
fn unicode_prefix_and_surrounding_suffix_spaces_are_preserved() {
    let prefix = "e\u{0301} USB ";
    let candidate = format!("{prefix} learning ");
    let live = context(prefix);
    let mut session = PredictionSession::default();
    session.observe(Some(live.clone()), 0);
    let request = session.poll(700).unwrap();
    let suffix = validate_and_extract_continuation(
        &request.context.prefix, &candidate, PredictionLanguage::English,
    );
    assert_eq!(suffix, Some(" learning "));
    assert!(session.complete(request.token, 800, suffix.is_some()));
    assert!(session.accept(request.token, &live));
    assert_eq!(format!("{}{}", live.prefix, suffix.unwrap()), candidate);
}

#[test]
fn valid_text_from_an_old_generation_is_not_published() {
    let live = context("今天晚上");
    let mut session = PredictionSession::default();
    session.observe(Some(live.clone()), 0);
    let request = session.poll(700).unwrap();
    session.observe(Some(context("今天晚")), 710);
    session.observe(Some(live.clone()), 720);
    let suffix = validate_and_extract_continuation(
        &request.context.prefix, "今天晚上一起吃飯", PredictionLanguage::TraditionalChinese,
    );
    assert!(suffix.is_some());
    assert!(!session.complete(request.token, 800, suffix.is_some()));
    assert!(!session.accept(request.token, &live));
}

#[test]
fn valid_ready_result_cannot_be_accepted_in_another_document() {
    let live = context("machine");
    let mut session = PredictionSession::default();
    session.observe(Some(live.clone()), 0);
    let request = session.poll(700).unwrap();
    let suffix = validate_and_extract_continuation(
        &request.context.prefix, "machine learning", PredictionLanguage::English,
    );
    assert!(session.complete(request.token, 800, suffix.is_some()));
    let mut other = live.clone();
    other.document_id += 1;
    assert!(!session.accept(request.token, &other));
    assert!(!session.accept(request.token, &live));
}
