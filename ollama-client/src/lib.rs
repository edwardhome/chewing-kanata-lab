// SPDX-License-Identifier: MIT
//! Bounded blocking HTTP transport for local Ollama loopback endpoint.
//!
//! Intended exclusively for background worker-thread execution, never the UI thread.
//!
//! # Invariants & Boundaries
//! - Fixed production endpoint: `http://127.0.0.1:11434`
//! - No redirects, no proxy, no cloud fallback
//! - Connect timeout <= 250ms, genuine total request/body deadline <= 2s
//! - Response header size bounded to 8 KiB (8,192 bytes)
//! - Response body hard capped at 16 KiB (16,384 bytes)
//! - Non-success HTTP statuses (4xx, 5xx) and redirects (3xx) rejected as typed errors
//! - Zero text/prompt logging in errors or debug formatters; errors use static categories
//!   to prevent leaking untrusted HTTP response payloads or parser diagnostics
//! - Cancellation checked before network initiation and before delivering results.
//!
//! # Cancellation Semantics
//! Since this transport relies on synchronous blocking socket I/O, cancellation tokens
//! are polled immediately prior to initiating connection and immediately before delivering
//! response bytes. In-flight blocking socket calls are bounded by the shared global deadline
//! (<= 2s) and will terminate when the socket deadline expires or data completes; immediate
//! mid-syscall cancellation is not supported by blocking sockets without socket closure.

use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
mod protocol_contract;

/// Fixed production endpoint for the local Ollama instance.
pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434";

/// Maximum allowed connection timeout (250 ms).
pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(250);

/// Maximum allowed total request and body deadline (2.0 s).
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(2);

/// Maximum permitted response headers size: 8 KiB (8,192 bytes).
pub const MAX_HEADER_SIZE: usize = 8 * 1024;

/// Maximum permitted response body size: 16 KiB (16,384 bytes).
pub const MAX_RESPONSE_SIZE: usize = 16 * 1024;

/// Strongly typed errors produced by the Ollama HTTP transport.
///
/// Error variants use static categories without raw error strings from I/O or
/// HTTP parser diagnostics, ensuring untrusted response text is never leaked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OllamaError {
    /// Request was cancelled via `CancellationToken` before initiation or delivery.
    Cancelled,
    /// Connection to local endpoint failed.
    Connect,
    /// Request exceeded the connect timeout or total global deadline.
    Timeout,
    /// An HTTP redirect was received and rejected.
    Redirect(u16),
    /// Server returned a non-success HTTP status code (not 200..=299).
    HttpStatus(u16),
    /// Response payload exceeded the 16 KiB limit.
    Oversized {
        limit: usize,
        actual: Option<usize>,
    },
    /// Low-level transport I/O error occurred.
    Io,
    /// HTTP protocol or parse violation.
    Protocol,
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "request cancelled"),
            Self::Connect => write!(f, "connection to endpoint failed"),
            Self::Timeout => write!(f, "request deadline exceeded"),
            Self::Redirect(code) => write!(f, "HTTP redirect rejected (status {code})"),
            Self::HttpStatus(code) => write!(f, "HTTP non-success status {code}"),
            Self::Oversized { limit, actual } => {
                write!(f, "response exceeded {limit} bytes (actual: {actual:?})")
            }
            Self::Io => write!(f, "transport I/O error"),
            Self::Protocol => write!(f, "HTTP protocol violation"),
        }
    }
}

impl std::error::Error for OllamaError {}

/// Thread-safe cancellation token.
///
/// Checked before network connection and before final result delivery.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Bounded blocking HTTP client for local Ollama.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    agent: ureq::Agent,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OllamaClient {
    /// Create a client targeting the fixed production endpoint `http://127.0.0.1:11434`.
    pub fn new() -> Self {
        Self::with_config(DEFAULT_OLLAMA_ENDPOINT, CONNECT_TIMEOUT, TOTAL_TIMEOUT)
    }

    fn with_config(base_url: &str, connect_timeout: Duration, total_timeout: Duration) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_connect(Some(connect_timeout))
            .timeout_global(Some(total_timeout))
            .timeout_recv_response(Some(total_timeout))
            .timeout_recv_body(Some(total_timeout))
            .timeout_send_body(Some(total_timeout))
            .max_response_header_size(MAX_HEADER_SIZE)
            .max_redirects(0)
            .max_redirects_will_error(true)
            .proxy(None)
            .http_status_as_error(false)
            .build()
            .into();

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            agent,
        }
    }

    /// Test-private constructor for local mock servers.
    ///
    /// Available only in `cfg(test)` and strictly enforces loopback endpoints.
    #[cfg(test)]
    pub(crate) fn new_for_test(base_url: &str) -> Self {
        assert!(
            base_url.starts_with("http://127.0.0.1:") || base_url.starts_with("http://localhost:"),
            "test client must target a loopback endpoint"
        );
        Self::with_config(base_url, CONNECT_TIMEOUT, TOTAL_TIMEOUT)
    }

    /// Test-private constructor allowing custom timeouts for fast deadline tests.
    #[cfg(test)]
    pub(crate) fn new_for_test_with_timeouts(
        base_url: &str,
        connect_timeout: Duration,
        total_timeout: Duration,
    ) -> Self {
        assert!(
            base_url.starts_with("http://127.0.0.1:") || base_url.starts_with("http://localhost:"),
            "test client must target a loopback endpoint"
        );
        Self::with_config(base_url, connect_timeout, total_timeout)
    }

    /// Query installed models via GET /api/tags.
    ///
    /// Returns raw response bytes bounded to 16 KiB.
    pub fn list_models(&self, cancel: &CancellationToken) -> Result<Vec<u8>, OllamaError> {
        if cancel.is_cancelled() {
            return Err(OllamaError::Cancelled);
        }

        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .agent
            .get(&url)
            .header("accept", "application/json")
            .call()
            .map_err(Self::map_ureq_error)?;

        self.read_response(resp, cancel)
    }

    /// Send phrase-prediction completion request via POST /api/chat.
    ///
    /// Caller provides serialized JSON bytes in `request_bytes`.
    /// Returns raw response bytes bounded to 16 KiB.
    pub fn chat(
        &self,
        request_bytes: &[u8],
        cancel: &CancellationToken,
    ) -> Result<Vec<u8>, OllamaError> {
        if cancel.is_cancelled() {
            return Err(OllamaError::Cancelled);
        }

        let url = format!("{}/api/chat", self.base_url);
        let resp = self
            .agent
            .post(&url)
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .send(request_bytes)
            .map_err(Self::map_ureq_error)?;

        self.read_response(resp, cancel)
    }

    fn read_response(
        &self,
        resp: ureq::http::Response<ureq::Body>,
        cancel: &CancellationToken,
    ) -> Result<Vec<u8>, OllamaError> {
        let status = resp.status().as_u16();
        if (300..=399).contains(&status) {
            return Err(OllamaError::Redirect(status));
        }
        if !(200..=299).contains(&status) {
            return Err(OllamaError::HttpStatus(status));
        }

        // Fast-path rejection if Content-Length header exceeds 16 KiB
        if let Some(content_length_str) = resp
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
        {
            if let Ok(content_length) = content_length_str.trim().parse::<usize>() {
                if content_length > MAX_RESPONSE_SIZE {
                    return Err(OllamaError::Oversized {
                        limit: MAX_RESPONSE_SIZE,
                        actual: Some(content_length),
                    });
                }
            }
        }

        let mut reader = resp.into_body().into_reader();
        let mut buf = Vec::new();
        let mut chunk = [0u8; 2048];

        loop {
            let n = match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    if e.kind() == io::ErrorKind::TimedOut {
                        return Err(OllamaError::Timeout);
                    }
                    return Err(OllamaError::Io);
                }
            };

            if buf.len() + n > MAX_RESPONSE_SIZE {
                return Err(OllamaError::Oversized {
                    limit: MAX_RESPONSE_SIZE,
                    actual: None,
                });
            }
            buf.extend_from_slice(&chunk[..n]);
        }

        // Post-read cancellation check before returning data to caller
        if cancel.is_cancelled() {
            return Err(OllamaError::Cancelled);
        }

        Ok(buf)
    }

    fn map_ureq_error(err: ureq::Error) -> OllamaError {
        match err {
            ureq::Error::Timeout(_) => OllamaError::Timeout,
            ureq::Error::StatusCode(code) => {
                if (300..=399).contains(&code) {
                    OllamaError::Redirect(code)
                } else {
                    OllamaError::HttpStatus(code)
                }
            }
            ureq::Error::TooManyRedirects | ureq::Error::RedirectFailed => {
                OllamaError::Redirect(300)
            }
            ureq::Error::ConnectionFailed | ureq::Error::HostNotFound => {
                OllamaError::Connect
            }
            ureq::Error::Io(e) => {
                if e.kind() == io::ErrorKind::TimedOut {
                    OllamaError::Timeout
                } else if e.kind() == io::ErrorKind::ConnectionRefused {
                    OllamaError::Connect
                } else {
                    OllamaError::Io
                }
            }
            ureq::Error::Protocol(_) => OllamaError::Protocol,
            _ => OllamaError::Io,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Local Mock-Server Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::thread;
    use tiny_http::{Header, Response, Server, StatusCode};

    fn spawn_server<F>(handler: F) -> (String, thread::JoinHandle<()>)
    where
        F: FnOnce(Server) + Send + 'static,
    {
        let server = Server::http("127.0.0.1:0").expect("bind local test server");
        let port = server.server_addr().to_ip().unwrap().port();
        let base_url = format!("http://127.0.0.1:{port}");
        let handle = thread::spawn(move || handler(server));
        (base_url, handle)
    }

    // ─────────────────────────── NORMAL RESPONSES ─────────────────────────────

    #[test]
    fn test_list_models_normal_success() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            assert_eq!(req.method().as_str(), "GET");
            assert_eq!(req.url(), "/api/tags");
            let body = r#"{"models":[{"name":"qwen2.5:0.5b"}]}"#;
            let resp = Response::from_string(body).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let res = client.list_models(&cancel).expect("successful models list");
        assert_eq!(res, br#"{"models":[{"name":"qwen2.5:0.5b"}]}"#);
        server_thread.join().unwrap();
    }

    #[test]
    fn test_chat_normal_success() {
        let (base_url, server_thread) = spawn_server(|server| {
            let mut req = server.recv().unwrap();
            assert_eq!(req.method().as_str(), "POST");
            assert_eq!(req.url(), "/api/chat");

            // Verify content-type header
            let ct = req
                .headers()
                .iter()
                .find(|h| {
                    let field: &str = h.field.as_str().as_ref();
                    field.eq_ignore_ascii_case("content-type")
                })
                .expect("content-type header present");
            let value: &str = ct.value.as_str().as_ref();
            assert_eq!(value, "application/json");

            // Verify request body bytes preserved
            let mut received_body = Vec::new();
            req.as_reader().read_to_end(&mut received_body).unwrap();
            assert_eq!(received_body, br#"{"prompt":"ping"}"#);

            let body = r#"{"response":"pong"}"#;
            let resp = Response::from_string(body).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let res = client
            .chat(br#"{"prompt":"ping"}"#, &cancel)
            .expect("successful chat");
        assert_eq!(res, br#"{"response":"pong"}"#);
        server_thread.join().unwrap();
    }

    // ─────────────────────── CHUNKED / FRAGMENTED ─────────────────────────────

    #[test]
    fn test_chunked_fragmented_response() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            let chunked_data = b"chunk1-chunk2-chunk3";
            let resp = Response::new(
                StatusCode(200),
                vec![Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()],
                Box::new(Cursor::new(chunked_data.to_vec())),
                None, // chunked
                None,
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let res = client.list_models(&cancel).expect("chunked response ok");
        assert_eq!(res, b"chunk1-chunk2-chunk3");
        server_thread.join().unwrap();
    }

    // ─────────────────────────── REDIRECTS ────────────────────────────────────

    #[test]
    fn test_rejects_302_redirect() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            let resp = Response::empty(302).with_header(
                Header::from_bytes(&b"Location"[..], &b"http://127.0.0.1:11434/other"[..]).unwrap(),
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("redirect rejected");
        assert_eq!(err, OllamaError::Redirect(302));
        server_thread.join().unwrap();
    }

    #[test]
    fn test_rejects_307_temporary_redirect() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            let resp = Response::empty(307).with_header(
                Header::from_bytes(&b"Location"[..], &b"http://127.0.0.1:11434/other"[..]).unwrap(),
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let err = client
            .chat(b"{}", &cancel)
            .expect_err("307 redirect rejected");
        match err {
            OllamaError::Redirect(code) => assert!(code == 307 || code == 300),
            other => panic!("expected Redirect error, got {other:?}"),
        }
        server_thread.join().unwrap();
    }

    // ───────────────────────── NON-SUCCESS STATUS ─────────────────────────────

    #[test]
    fn test_rejects_404_not_found() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            req.respond(Response::empty(404)).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("404 rejected");
        assert_eq!(err, OllamaError::HttpStatus(404));
        server_thread.join().unwrap();
    }

    #[test]
    fn test_rejects_500_internal_server_error() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            req.respond(Response::empty(500)).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let err = client
            .chat(b"{}", &cancel)
            .expect_err("500 rejected");
        assert_eq!(err, OllamaError::HttpStatus(500));
        server_thread.join().unwrap();
    }

    // ──────────────────────── OVERSIZED RESPONSE ──────────────────────────────

    #[test]
    fn test_rejects_oversized_content_length() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            let resp = Response::new(
                StatusCode(200),
                vec![],
                Box::new(Cursor::new(vec![b'x'; 16385])),
                Some(16385),
                None,
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("oversized rejected");
        assert_eq!(
            err,
            OllamaError::Oversized {
                limit: MAX_RESPONSE_SIZE,
                actual: Some(16385),
            }
        );
        server_thread.join().unwrap();
    }

    #[test]
    fn test_rejects_oversized_chunked_stream() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            let big_data = vec![b'y'; 18000];
            let resp = Response::new(
                StatusCode(200),
                vec![],
                Box::new(Cursor::new(big_data)),
                None, // chunked
                None,
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("oversized chunked");
        assert_eq!(
            err,
            OllamaError::Oversized {
                limit: MAX_RESPONSE_SIZE,
                actual: None,
            }
        );
        server_thread.join().unwrap();
    }

    #[test]
    fn test_accepts_exact_16k_response() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            let exact_data = vec![b'z'; MAX_RESPONSE_SIZE];
            let resp = Response::new(
                StatusCode(200),
                vec![],
                Box::new(Cursor::new(exact_data)),
                Some(MAX_RESPONSE_SIZE),
                None,
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let res = client.list_models(&cancel).expect("exact 16k accepted");
        assert_eq!(res.len(), MAX_RESPONSE_SIZE);
        server_thread.join().unwrap();
    }

    // ──────────────────────── TIMEOUT / DEADLINE ──────────────────────────────

    #[test]
    fn test_stalled_body_deadline_exceeded() {
        struct StalledReader;
        impl Read for StalledReader {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                thread::sleep(Duration::from_millis(600));
                Ok(0)
            }
        }

        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            let resp = Response::new(
                StatusCode(200),
                vec![],
                Box::new(StalledReader),
                None,
                None,
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test_with_timeouts(
            &base_url,
            Duration::from_millis(100),
            Duration::from_millis(200),
        );
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("stalled timeout");
        assert_eq!(err, OllamaError::Timeout);
        server_thread.join().unwrap();
    }

    #[test]
    fn test_slow_drip_body_deadline_exceeded() {
        struct SlowDripReader {
            chunks_remaining: usize,
            chunk_delay: Duration,
        }
        impl Read for SlowDripReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                if self.chunks_remaining == 0 {
                    return Ok(0);
                }
                thread::sleep(self.chunk_delay);
                self.chunks_remaining -= 1;
                let payload = b"drip";
                let n = payload.len().min(buf.len());
                buf[..n].copy_from_slice(&payload[..n]);
                Ok(n)
            }
        }

        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            // 8 chunks with 50ms delay = 400ms total drip time
            let resp = Response::new(
                StatusCode(200),
                vec![],
                Box::new(SlowDripReader {
                    chunks_remaining: 8,
                    chunk_delay: Duration::from_millis(50),
                }),
                None,
                None,
            );
            req.respond(resp).unwrap();
        });

        // Client configured with 150ms total deadline; should time out during slow drip
        let client = OllamaClient::new_for_test_with_timeouts(
            &base_url,
            Duration::from_millis(100),
            Duration::from_millis(150),
        );
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("slow drip timeout");
        assert_eq!(err, OllamaError::Timeout);
        server_thread.join().unwrap();
    }

    #[test]
    fn test_shared_global_deadline_header_plus_body() {
        struct DelayedBodyReader {
            delay: Duration,
            done: bool,
        }
        impl Read for DelayedBodyReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                if self.done {
                    return Ok(0);
                }
                thread::sleep(self.delay);
                self.done = true;
                let payload = b"done";
                let n = payload.len().min(buf.len());
                buf[..n].copy_from_slice(&payload[..n]);
                Ok(n)
            }
        }

        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            // Delay 120ms before writing headers
            thread::sleep(Duration::from_millis(120));
            // Body read will delay another 120ms (total = 240ms)
            let resp = Response::new(
                StatusCode(200),
                vec![],
                Box::new(DelayedBodyReader {
                    delay: Duration::from_millis(120),
                    done: false,
                }),
                None,
                None,
            );
            req.respond(resp).unwrap();
        });

        // Client deadline is 200ms.
        // Header time (120ms) < 200ms.
        // Body time (120ms) < 200ms.
        // Combined time (240ms) > 200ms global deadline!
        let client = OllamaClient::new_for_test_with_timeouts(
            &base_url,
            Duration::from_millis(100),
            Duration::from_millis(200),
        );
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("shared deadline timeout");
        assert_eq!(err, OllamaError::Timeout);
        server_thread.join().unwrap();
    }

    // ──────────────────────── HEADER SIZE BOUND ───────────────────────────────

    #[test]
    fn test_rejects_oversized_header() {
        let (base_url, server_thread) = spawn_server(|server| {
            let req = server.recv().unwrap();
            // Generate header exceeding MAX_HEADER_SIZE (8 KiB)
            let big_header_val = vec![b'a'; 9000];
            let resp = Response::empty(200).with_header(
                Header::from_bytes(&b"X-Custom-Header"[..], big_header_val).unwrap(),
            );
            req.respond(resp).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("oversized header rejected");
        match err {
            OllamaError::Protocol | OllamaError::Io => {}
            other => panic!("expected Protocol or Io error for oversized header, got {other:?}"),
        }
        server_thread.join().unwrap();
    }

    // ────────────────────────── CANCELLATION ──────────────────────────────────

    #[test]
    fn test_cancellation_before_send() {
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(cancel.is_cancelled());

        let client = OllamaClient::new_for_test("http://127.0.0.1:9");
        let err = client.list_models(&cancel).expect_err("cancelled before send");
        assert_eq!(err, OllamaError::Cancelled);

        let err_chat = client.chat(b"{}", &cancel).expect_err("chat cancelled");
        assert_eq!(err_chat, OllamaError::Cancelled);
    }

    #[test]
    fn test_cancellation_before_deliver() {
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();

        let (base_url, server_thread) = spawn_server(move |server| {
            let req = server.recv().unwrap();
            cancel_clone.cancel();
            req.respond(Response::from_string("ok")).unwrap();
        });

        let client = OllamaClient::new_for_test(&base_url);
        let err = client.list_models(&cancel).expect_err("cancelled before deliver");
        assert_eq!(err, OllamaError::Cancelled);
        server_thread.join().unwrap();
    }

    // ──────────────────────── CONNECTION REFUSED ──────────────────────────────

    #[test]
    fn test_connection_refused_unbound_port() {
        let client = OllamaClient::new_for_test_with_timeouts(
            "http://127.0.0.1:49999",
            Duration::from_millis(50),
            Duration::from_millis(100),
        );
        let cancel = CancellationToken::new();
        let err = client.list_models(&cancel).expect_err("connection fails");
        match err {
            OllamaError::Connect | OllamaError::Timeout => {}
            other => panic!("expected Connect or Timeout, got {other:?}"),
        }
    }
}
