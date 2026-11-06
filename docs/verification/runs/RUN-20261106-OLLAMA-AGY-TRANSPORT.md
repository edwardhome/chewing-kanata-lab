# RUN-20261106-OLLAMA-AGY-TRANSPORT

狀態：PASS  
時間：2026-11-06T10:55:00+08:00  
分支：feature/ollama-phrase-prediction  
作者/提交者：AGY <ckai-dev@local>  
範圍：`ollama-client/`（純獨立 transport crate）與本驗證報告

---

## 審查回饋修正摘要

針對審查意見完成以下防護與驗證升級：

1. **移除錯誤中所有動態字串與 Parser 診斷**：
   - 原始 `Connect(String)`、`Timeout(String)`、`Io(String)`、`Protocol(String)` 全面改為靜態類別（無堆疊字串欄位），防止 HTTP Parser 報錯或底層 socket 錯誤中夾帶未受信任的伺服器回傳內文。
   - `OllamaError` 派生 `Copy, Clone, PartialEq, Eq`，Display 輸出僅包含固定靜態描述文字。
2. **回應標頭大小限制（Bounded Header Size）**：
   - 設定 `max_response_header_size(8 * 1024)`（8 KiB），防範 Header-bomb 與惡意超大標頭消耗記憶體。
3. **慢速滴流（Slow Drip）內文總期限測試**：
   - 新增 `test_slow_drip_body_deadline_exceeded`：模擬伺服器以 50ms 間隔極慢速分段發送資料（未完全停頓，但累積時長超出總期限），驗證 `timeout_global` 正確中斷並回傳 `OllamaError::Timeout`。
4. **標頭與內文共用單一總期限測試**：
   - 新增 `test_shared_global_deadline_header_plus_body`：伺服器標頭延遲 120ms（< 200ms）、內文讀取延遲 120ms（< 200ms），單項均未超時，但兩者累計 240ms > 200ms 總期限。驗證客戶端共用同一總預算，於內文讀取時正確逾時返回 `OllamaError::Timeout`。

---

## 模組架構與公用介面

```rust
pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434";
pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(250);
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(2);
pub const MAX_HEADER_SIZE: usize = 8 * 1024;    // 8 KiB
pub const MAX_RESPONSE_SIZE: usize = 16 * 1024; // 16 KiB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OllamaError {
    Cancelled,
    Connect,
    Timeout,
    Redirect(u16),
    HttpStatus(u16),
    Oversized { limit: usize, actual: Option<usize> },
    Io,
    Protocol,
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

    #[cfg(test)]
    pub(crate) fn new_for_test(base_url: &str) -> Self;
    #[cfg(test)]
    pub(crate) fn new_for_test_with_timeouts(base_url: &str, connect_timeout: Duration, total_timeout: Duration) -> Self;
}
```

### 規格遵守對照表

| 規格要求 | 實作方式 | 狀態 |
| :--- | :--- | :---: |
| 固定生產端點 | `http://127.0.0.1:11434`，零雲端、無 fallback | ✅ |
| 禁止任意 URL 公開建構 | 生產建構子 `new()` 固定端點；動態測試建構子 `new_for_test*` 標記 `#[cfg(test)] pub(crate)` 且強制斷言 loopback | ✅ |
| 連線超時 <= 250ms | `timeout_connect(Some(Duration::from_millis(250)))` | ✅ |
| 整體總期限 <= 2s | `timeout_global(Some(Duration::from_secs(2)))`，標頭與內文串流共用單一預算 | ✅ |
| 8 KiB 標頭上限 | `max_response_header_size(8 * 1024)` | ✅ |
| 16 KiB 內文上限 | 讀取時以 16,384 bytes 為硬上限；Content-Length 預先檢查及 Chunked 讀取中即時截斷拒絕 | ✅ |
| 拒絕重導向 (Redirect) | `max_redirects(0)` + `max_redirects_will_error(true)`，回傳 `OllamaError::Redirect` | ✅ |
| 拒絕非成功 HTTP 狀態 | 非 200..=299 狀態碼回傳 `OllamaError::HttpStatus(code)` | ✅ |
| 零文字日誌與靜態錯誤 | `OllamaError` 採靜態列舉類別，無任何文字欄位，禁止洩漏伺服器回覆與解析文字 | ✅ |
| 支援 Chunked/Fragmented | 完整支援 HTTP/1.1 chunked 傳輸解包 | ✅ |
| 慢速滴流防護 | 即使每 chunk 持續有位元組抵達，超出總期限即刻終止 | ✅ |
| 取消機制語意 | 於連線前與回傳結果前雙重檢查 `CancellationToken`；若於阻塞 syscall 執行中由總期限保護 | ✅ |

---

## 本機假伺服器測試結果

測試工具：`tiny_http` 本機隨機 loopback port（`127.0.0.1:0`）

**測試執行命令：**
```powershell
$env:PATH += ';' + $env:USERPROFILE + '\.cargo\bin'
cargo test --manifest-path ollama-client/Cargo.toml --locked
```

**測試輸出：**
```text
running 17 tests
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
test tests::test_rejects_oversized_header ... ok
test tests::test_shared_global_deadline_header_plus_body ... ok
test tests::test_slow_drip_body_deadline_exceeded ... ok
test tests::test_stalled_body_deadline_exceeded ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
```
**退出碼（Exit Code）：** 0

### 測試覆蓋項目清單（17 個測試）

1. `test_list_models_normal_success`：驗證 `GET /api/tags` 正常 200 回應與 JSON 位元組。
2. `test_chat_normal_success`：驗證 `POST /api/chat` 包含 `Content-Type: application/json` 標頭且位元組原樣傳輸。
3. `test_chunked_fragmented_response`：驗證分段 chunked 串流回應之完整重組。
4. `test_rejects_302_redirect`：驗證 302 重導向立即拒絕並回傳 `OllamaError::Redirect(302)`。
5. `test_rejects_307_temporary_redirect`：驗證 307 臨時重導向被安全拒絕。
6. `test_rejects_404_not_found`：驗證 404 狀態碼回傳 `OllamaError::HttpStatus(404)`。
7. `test_rejects_500_internal_server_error`：驗證 500 伺服器錯誤回傳 `OllamaError::HttpStatus(500)`。
8. `test_rejects_oversized_content_length`：宣告 Content-Length 16,385 bytes（超限 1 byte）於標頭解析階段即時拒絕。
9. `test_rejects_oversized_chunked_stream`：無 Content-Length 之 Chunked 串流在累積超過 16 KiB 時即刻終止並報錯。
10. `test_accepts_exact_16k_response`：驗證剛好 16,384 bytes（邊界值）正常讀取成功。
11. `test_stalled_body_deadline_exceeded`：伺服器於標頭發送後暫停發送內文，驗證客戶端於期限內逾時並回傳 `OllamaError::Timeout`。
12. `test_slow_drip_body_deadline_exceeded`：伺服器以 50ms 間隔持續慢速滴流傳送小區塊，累計超過總體期限，驗證正確觸發 `OllamaError::Timeout`。
13. `test_shared_global_deadline_header_plus_body`：標頭與內文各自延遲 120ms（個別均小於 200ms），但累計 240ms > 200ms 總期限，驗證共用單一預算並超時返回 `OllamaError::Timeout`。
14. `test_rejects_oversized_header`：發送超過 8 KiB 之超大標頭，驗證客戶端安全攔截。
15. `test_cancellation_before_send`：連線前取消，完全不發起網路通訊直接回傳 `OllamaError::Cancelled`。
16. `test_cancellation_before_deliver`：伺服器回應後但在遞交結果前取消，回傳 `OllamaError::Cancelled`。
17. `test_connection_refused_unbound_port`：未綁定之本機 port 連線失敗測試。

---

## 既有模組回歸驗證

```powershell
cargo test --manifest-path policy-core/Cargo.toml --locked
```
**結果：** 88 passed (unit) + 5 passed (contract)，0 failed，退出碼 0。
