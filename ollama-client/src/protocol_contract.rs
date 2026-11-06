// SPDX-License-Identifier: MIT
use super::{CancellationToken, OllamaClient};
use chewing_policy_core::{ollama_protocol::{select_local_model, build_prediction_request,
    parse_prediction_response}, prediction::PredictionLanguage,
    prediction_session::{PredictionContext, PredictionSession, Safety}};
use serde_json::json;
use tiny_http::{Response, Server};

fn context(prefix: &str) -> PredictionContext {
    PredictionContext { prefix: prefix.into(), focus_id: 1, document_id: 1, mode_id: 1,
        selection_start: prefix.chars().count(), selection_len: 0, complete: true,
        enabled: true, safety: Safety::Safe }
}

fn run_pipeline(reply: &str, edit_while_pending: bool) -> (Option<String>, bool) {
    let server = Server::http("127.0.0.1:0").unwrap();
    let url = format!("http://127.0.0.1:{}", server.server_addr().to_ip().unwrap().port());
    let reply = reply.to_owned();
    let thread = std::thread::spawn(move || {
        let tags = server.recv_timeout(std::time::Duration::from_secs(3)).unwrap().unwrap();
        assert_eq!(tags.url(), "/api/tags");
        tags.respond(Response::from_string(json!({"models":[{"name":"test:local","size":100,
            "details":{"format":"gguf"}}]}).to_string())).unwrap();
        let mut chat = server.recv_timeout(std::time::Duration::from_secs(3)).unwrap().unwrap();
        assert_eq!(chat.url(), "/api/chat");
        let mut request = String::new(); chat.as_reader().read_to_string(&mut request).unwrap();
        let request: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(request["stream"], false);
        assert_eq!(request["model"], "test:local");
        chat.respond(Response::from_string(json!({"model":"test:local","done":true,
            "done_reason":"stop","message":{"role":"assistant",
                "content":json!({"suggestions":[reply]}).to_string()}}).to_string())).unwrap();
    });
    let client = OllamaClient::new_for_test(&url); let cancel = CancellationToken::new();
    let model = select_local_model(&client.list_models(&cancel).unwrap(), "test:local").unwrap();
    let live = context("今天晚上"); let mut session = PredictionSession::default();
    session.observe(Some(live.clone()), 0); let request = session.poll(700).unwrap();
    let bytes = build_prediction_request(&model, &request.context.prefix, PredictionLanguage::TraditionalChinese, false).unwrap();
    if edit_while_pending { session.observe(Some(context("今天晚")), 710); session.observe(Some(live.clone()), 720); }
    let response = client.chat(&bytes, &cancel).unwrap();
    let suffix = parse_prediction_response(&response, &model, &request.context.prefix, PredictionLanguage::TraditionalChinese).unwrap();
    let ready = session.complete(request.token, 800, suffix.is_some());
    assert_eq!(live.prefix, "今天晚上");
    let accepted = session.accept(request.token, &live);
    assert_eq!(accepted, ready);
    assert!(!session.accept(request.token, &live));
    thread.join().unwrap();
    (suffix, accepted)
}

#[test]
fn http_json_validation_and_explicit_acceptance_work_together() {
    let (suffix, accepted) = run_pipeline("今天晚上一起吃飯", false);
    assert_eq!(suffix.as_deref(), Some("一起吃飯"));
    assert!(accepted);
}

#[test]
fn successful_http_does_not_bypass_prefix_or_generation_checks() {
    let (suffix, accepted) = run_pipeline("明天晚上一起吃飯", false);
    assert!(suffix.is_none()); assert!(!accepted);
    let (suffix, accepted) = run_pipeline("今天晚上一起吃飯", true);
    assert!(suffix.is_some()); assert!(!accepted);
}
