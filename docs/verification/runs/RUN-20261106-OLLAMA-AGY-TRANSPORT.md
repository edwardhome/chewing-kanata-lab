# RUN-20261106-OLLAMA-AGY-TRANSPORT

狀態：PASS  
時間：2026-11-06T10:45:00+08:00  
分支：feature/ollama-phrase-prediction  
作者/提交者：AGY <ckai-dev@local>  
範圍：`ollama-client/`（純獨立 transport crate）與本驗證報告

---

## 實作摘要

為 Ollama 本機詞組預測提供一個有邊界、阻斷式（blocking）的純 HTTP Transport 模組，專供背景 Worker Thread 使用，嚴禁於 UI/TSF 執行緒調用。

### 模組架構與公用介面

```rust
pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434";
pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(250);
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(2);
pub const MAX_RESPONSE_SIZE: usize = 16 * 1024; // 16 KiB

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OllamaError {
    Cancelled,
    Connect(String),
    Timeout(String),
    Redirect(u16),
    HttpStatus(u16),
    Oversized { limit: usize, actual: Option<usize> },
    Io(String),
    Protocol(String),
}

pub struct CancellationToken { ... }
impl CancellationToken {
    pub fn new() -> Self;
    pub fn cancel(&self);
    pub fn is_cancelled(&self) -> bool;
}

pub struct OllamaClient { ... }
impl OllamaClient {
    pub fn new() -> Self;
    pub fn list_models(&self, cancel: &CancellationToken) -> Result<Vec<u8>, OllamaError>;
    pub fn chat(&self, request_bytes: &[u8], cancel: &CancellationToken) -> Result<Vec<u8>, OllamaError>;
}
```

### 規格遵守對照表

| 規格要求 | 實作方式 | 狀態 |
| :--- | :--- | :---: |
| 固定生產端點 | `http://127.0.0.1:11434`，零雲端、無 fallback | ✅ |
| 禁止任意 URL 公開建構 | 生產建構子 `new()` 固定端點；動態測試建構子 `new_for_test*` 標記 `#[cfg(test)] pub(crate)` 且強制斷言 loopback | ✅ |
| 連線超時 <= 250ms | `timeout_connect(Some(Duration::from_millis(250)))` | ✅ |
| 整體總期限 <= 2s | `timeout_global(Some(Duration::from_secs(2)))` + 讀寫與接收階段總期限保護 | ✅ |
| 16 KiB 回應上限 | 讀取時以 16,384 bytes 為硬上限；Content-Length 預先檢查及 Chunked 讀取中即時截斷拒絕 | ✅ |
| 拒絕重導向 (Redirect) | `max_redirects(0)` + `max_redirects_will_error(true)`，回傳 `OllamaError::Redirect` | ✅ |
| 拒絕非成功 HTTP 狀態 | 非 200..=299 狀態碼回傳 `OllamaError::HttpStatus(code)` | ✅ |
| 零使用者文字日誌 | 錯誤型別 `OllamaError` 與 `Display` 格式化嚴禁印出提示詞或回覆內文 | ✅ |
| 支援 Chunked/Fragmented | 完整支援 HTTP/1.1 chunked 傳輸解包 | ✅ |
| 取消機制語意 | 於連線前與回傳結果前雙重檢查 `CancellationToken` | ✅ |

### 取消機制與阻塞 I/O 限制說明
本 Transport 使用標準 Rust 同步阻塞式通訊（`ureq`），專為背景工作執行緒設計。
- **取消檢查點**：在建立連線前、以及在讀取完整回應並解包後準備遞交結果前，均會檢查取消標記。
- **執行中 I/O 限制**：若於 socket 讀寫進行中取消，由於底層為阻塞式系統調用，無法在無關閉 socket 的情況下瞬間終止 syscall，此時由 <= 2 秒之整體總期限保證工作者必定於期限內逾時返回，絕不宣稱具有「任意瞬間中斷」能力。

---

## 本機假伺服器測試結果

測試工具：`tiny_http` 本機隨機 loopback port（`127.0.0.1:0`）

**測試執行命令：**
```powershell
$env:PATH += ';' + $env:USERPROFILE + '\.cargo\bin'
cargo test --manifest-path ollama-client/Cargo.toml
```

**測試輸出：**
```text
running 14 tests
test tests::test_accepts_exact_16k_response ... ok
test tests::test_cancellation_before_deliver ... ok
test tests::test_cancellation_before_send ... ok
test tests::test_chat_normal_success ... ok
test tests::test_chunked_fragmented_response ... ok
test tests::test_connection_refused_unbound_port ... ok
test tests::test_list_models_normal_success ... ok
test tests::test_rejects_302_redirect ... ok
test tests::test_rejects_307_temporary_redirect ... ok
test tests::test_rejects_404_not_found ... ok
test tests::test_rejects_500_internal_server_error ... ok
test tests::test_rejects_oversized_chunked_stream ... ok
test tests::test_rejects_oversized_content_length ... ok
test tests::test_stalled_body_deadline_exceeded ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
```
**退出碼（Exit Code）：** 0

### 測試覆蓋項目清單

1. `test_list_models_normal_success`：驗證 `GET /api/tags` 正常 200 回應與 JSON 位元組。
2. `test_chat_normal_success`：驗證 `POST /api/chat` 包含 `Content-Type: application/json` 標頭且位元組原樣送達與回傳。
3. `test_chunked_fragmented_response`：驗證分段 chunked 串流回應之完整重組。
4. `test_rejects_302_redirect`：驗證 302 重導向立即拒絕並回傳 `OllamaError::Redirect(302)`。
5. `test_rejects_307_temporary_redirect`：驗證 307 臨時重導向被安全拒絕。
6. `test_rejects_404_not_found`：驗證 404 狀態碼回傳 `OllamaError::HttpStatus(404)`。
7. `test_rejects_500_internal_server_error`：驗證 500 伺服器錯誤回傳 `OllamaError::HttpStatus(500)`。
8. `test_rejects_oversized_content_length`：宣告 Content-Length 16,385 bytes（超限 1 byte）於標頭解析階段即時拒絕。
9. `test_rejects_oversized_chunked_stream`：無 Content-Length 之 Chunked 串流在累積超過 16 KiB 時即刻終止並報錯。
10. `test_accepts_exact_16k_response`：驗證剛好 16,384 bytes（邊界值）正常讀取成功。
11. `test_stalled_body_deadline_exceeded`：伺服器於標頭發送後暫停發送內文，驗證客戶端於期限內逾時並回傳 `OllamaError::Timeout`。
12. `test_cancellation_before_send`：連線前取消，完全不發起網路通訊直接回傳 `OllamaError::Cancelled`。
13. `test_cancellation_before_deliver`：伺服器回應後但在遞交結果前取消，回傳 `OllamaError::Cancelled`。
14. `test_connection_refused_unbound_port`：未綁定之本機 port 連線失敗測試。

---

## 既有模組驗證

同時執行既有 `policy-core` 驗證：
```powershell
cargo test --manifest-path policy-core/Cargo.toml --locked
```
**結果：** 88 passed (unit) + 5 passed (contract)，0 failed，退出碼 0。未異動任何既有核心檔案。
