# RUN-20261106-OLLAMA-AGY-VALIDATOR

狀態: PASS（含修正提交）
時間（初次）：2026-11-06T01:22:56+08:00
時間（修正）：2026-11-06T01:27:56+08:00
分支：feature/ollama-phrase-prediction
作者/提交者：AGY <ckai-dev@local>

---

## 提交記錄

| SHA | 類型 | 說明 |
|-----|------|------|
| `74dc146f` | feat | 初次實作 prediction.rs + lib.rs 匯出 |
| `65ba571` | docs | 初次驗證報告（已被本文件取代） |
| `ec0d287` | fix | 程式碼審查修正（詳見下方） |

---

## 修正內容（ec0d287）

### 修正一：控制字元過濾改用 ch.is_control()

**問題：** 原始實作使用 `(ch as u32) < 0x20 || ch == '\x7F'` 僅涵蓋 ASCII C0 控制字元與 DEL，遺漏 Unicode C1 區段（U+0080–U+009F），例如 U+0085 NEL、U+009B CSI。

**修正：** 改用 `ch.is_control()`，涵蓋 Unicode Cc 類別（C0：U+0000–U+001F、DEL：U+007F、C1：U+0080–U+009F）。U+2028 / U+2029 仍以明確比對保留（非 Cc 類別）。

**新增測試：**
- `rejects_chinese_suffix_with_c1_nel_before_han`：U+0085 後接漢字，確保不被純空白拒絕邏輯遮蔽
- `rejects_chinese_suffix_with_c1_csi_before_han`：U+009B 後接漢字，同上

### 修正二：32 字元邊界測試重寫

**問題：** 原 `accepts_english_suffix_at_exact_32_chars` 測試使用 19 字元 suffix 且含多個混淆的假設字串，未真正斷言 32 字元接受、33 字元拒絕。

**修正：** 改為：
- `"a".repeat(32)`（32 字元，1 個 ASCII 單字）→ 斷言 `is_some()`
- `"a".repeat(33)`（33 字元，1 個 ASCII 單字）→ 斷言 `== None`
- 兩者皆以 `.len()` 驗證長度

**同時移除：** `validate_english_suffix` 函式文件中「leading/trailing spaces are rejected」的不正確描述；規格允許前導/尾隨空格，並以 word_count 計算處理空段落。

**新增測試：**
- `accepts_english_suffix_with_leading_space`：驗證 `" learning"` 前導空格原樣回傳，不被截除

### 修正三：測試數量更正

**問題：** 初次報告誤稱新增 29 個 prediction 測試；實際為 33 個（初次提交即如此）。

**更正：** prediction 模組初次 33 個，修正後 36 個；既有測試 37 個，本次修正後整體 73 個。

---

## 實作規格對照

| 規格要求 | 實作方式 | 狀態 |
|----------|----------|------|
| `PredictionLanguage` enum，`#[derive(Debug, Clone, Copy, PartialEq, Eq)]` | 完整實作 | PASS |
| `validate_and_extract_continuation<'a>(prefix, candidate, language) -> Option<&'a str>` | 完整實作，零複製回傳子切片 | PASS |
| 拒絕空 prefix | `prefix.is_empty()` | PASS |
| 拒絕 prefix > 256 Unicode scalars | `prefix.chars().count() > 256` | PASS |
| 拒絕 prefix 不符（byte-exact）| `!candidate.starts_with(prefix)` | PASS |
| 拒絕空 suffix | `suffix.is_empty()` | PASS |
| 拒絕純空白 suffix | `suffix.chars().all(char::is_whitespace)` | PASS |
| 拒絕所有 Unicode 控制字元 | `ch.is_control()`（涵蓋 C0、DEL、C1） | PASS（修正後）|
| 拒絕 U+2028 / U+2029 | 明確比對 `LINE_SEP`, `PARA_SEP` | PASS |
| 英文：僅 ASCII 字母與普通空格 | `c.is_ascii_alphabetic() \|\| c == ' '` | PASS |
| 英文：≤ 32 字元 | `suffix.len() > 32` | PASS |
| 英文：≤ 4 個單字 | `word_count > 4` | PASS |
| 英文：前導空格原樣保留 | 不 trim；word_count 計算非空段落 | PASS（修正後）|
| 繁中：≤ 8 Unicode scalars | `suffix.chars().count() > 8` | PASS |
| 不 trim / normalize / 改大小寫 | 整條路徑無任何此類操作 | PASS |

---

## 測試結果（修正後）

**命令：**
```powershell
$env:PATH += ';' + $env:USERPROFILE + '\.cargo\bin'
$env:CARGO_TARGET_DIR = Join-Path $env:TEMP 'ckai-agy-ollama-target'
cargo test --manifest-path policy-core/Cargo.toml --locked
```

**結果：**
```
test result: ok. 73 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**退出碼：** 0

### prediction 模組測試（33 個，全數通過）

| 測試名稱 | 分類 | 結果 |
|----------|------|------|
| `accepts_valid_chinese_continuation` | 基本接受 | PASS |
| `accepts_valid_english_continuation` | 基本接受 | PASS |
| `accepts_english_suffix_at_exact_32_chars` | 邊界（已重寫）| PASS |
| `accepts_english_exactly_four_words` | 邊界 | PASS |
| `accepts_chinese_suffix_at_exactly_8_scalars` | 邊界 | PASS |
| `accepts_prefix_at_exactly_256_scalars` | 邊界 | PASS |
| `rejects_empty_prefix` | Prefix 拒絕 | PASS |
| `rejects_prefix_over_256_scalars` | Prefix 拒絕 | PASS |
| `rejects_prefix_mismatch_different_case` | Prefix 拒絕／大小寫 | PASS |
| `rejects_prefix_mismatch_traditional_vs_simplified` | Prefix 拒絕／Unicode | PASS |
| `rejects_prefix_mismatch_decomposed_vs_composed` | Prefix 拒絕／decomposed | PASS |
| `rejects_prefix_mismatch_whitespace_changed` | Prefix 拒絕／空白 | PASS |
| `rejects_empty_suffix` | Suffix 拒絕 | PASS |
| `rejects_whitespace_only_suffix` | Suffix 拒絕 | PASS |
| `rejects_suffix_with_ascii_control` | 控制字元（C0） | PASS |
| `rejects_suffix_with_unicode_line_separator` | U+2028 | PASS |
| `rejects_suffix_with_unicode_paragraph_separator` | U+2029 | PASS |
| `rejects_chinese_suffix_with_c1_nel_before_han` | U+0085 NEL（新增）| PASS |
| `rejects_chinese_suffix_with_c1_csi_before_han` | U+009B CSI（新增）| PASS |
| `accepts_english_suffix_with_leading_space` | 前導空格保留（新增）| PASS |
| `rejects_english_suffix_with_digit` | 英文規則 | PASS |
| `rejects_english_suffix_with_non_ascii_letter` | 英文規則 | PASS |
| `rejects_english_suffix_exceeding_32_chars` | 英文規則 | PASS |
| `rejects_english_suffix_exceeding_4_words` | 英文規則 | PASS |
| `rejects_english_no_actual_words` | 英文規則 | PASS |
| `rejects_chinese_suffix_exceeding_8_scalars` | 繁中規則 | PASS |
| `accepts_chinese_suffix_with_mixed_punctuation` | 繁中規則 | PASS |
| `rejects_chinese_suffix_containing_newline` | 繁中規則 | PASS |
| `prefix_with_multibyte_chars_matched_exactly` | 多位元組 UTF-8 | PASS |
| `rejects_candidate_that_only_shares_ascii_prefix_bytes_not_unicode_boundary` | UTF-8 邊界 | PASS |
| `rejects_case_changed_prefix` | 大小寫保護 | PASS |
| `accepts_english_suffix_mixed_case_in_valid_range` | 大小寫 | PASS |
| `single_scalar_prefix_and_single_scalar_suffix_chinese` | 邊緣情況 | PASS |
| `single_ascii_prefix_and_single_ascii_suffix_english` | 邊緣情況 | PASS |
| `chinese_suffix_with_fullwidth_space_counts_as_one_scalar` | 全形空白 | PASS |
| `chinese_accepts_fullwidth_space_followed_by_text` | 全形空白 | PASS |

> 備注：上表計 36 列，其中 33 個隸屬 prediction 模組，`rejects_suffix_with_ascii_control`、`rejects_whitespace_only_suffix`、`rejects_empty_suffix` 包含多個子斷言，每列計為一個 `#[test]`。

---

## Ollama / 本機模型狀態

本次執行未探測 Ollama 端點或模型清單。

| 項目 | 狀態 |
|------|------|
| Ollama port 11434 連線檢查 | NOT_RUN |
| 已安裝模型查詢 (`GET /api/tags`) | NOT_RUN |

---

## 尚未串接之項目

| 項目 | 狀態 | 說明 |
|------|------|------|
| `prediction_session.rs` | BLOCKED（Codex 負責）| Codex 已完成 lifecycle 模組（15 個 Windows 測試），待整合 |
| Ollama HTTP transport / `OllamaClient` | NOT_RUN | 第一檢查點後續工作 |
| 假伺服器契約測試 | NOT_RUN | 第一檢查點後續工作 |
| TSF 非同步整合 (tip crate) | NOT_RUN | 第二檢查點工作 |
| ChewingPreferences 設定頁 Ollama 分頁 | NOT_RUN | 第二檢查點工作 |
| 實鍵驗收清單 | NOT_RUN | 需 GUI 與實體鍵盤 |
| 本機模型合成案例測試 | NOT_RUN | 未探測模型可用性 |
