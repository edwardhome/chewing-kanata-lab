# RUN-20261106-OLLAMA-AGY-VALIDATOR

ステータス: PASS  
日時: 2026-11-06T01:22:56+08:00  
分支: feature/ollama-phrase-prediction  
作者/提交者: AGY <ckai-dev@local>  
Commit: `74dc146f622a322f24361c4f7eefea5b54c8beb2`

---

## 實作摘要

### 新建檔案

| 路徑 | 說明 |
|------|------|
| `policy-core/src/prediction.rs` | `PredictionLanguage` enum + `validate_and_extract_continuation` fn |
| *(修改)* `policy-core/src/lib.rs` | 增加 `pub mod prediction` 與 `pub use` 匯出 |

### 實作規格對照

| 規格要求 | 實作方式 | 狀態 |
|----------|----------|------|
| `PredictionLanguage` enum (TraditionalChinese, English), `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` | 完整實作 | ✅ |
| `validate_and_extract_continuation<'a>(prefix, candidate, language) -> Option<&'a str>` | 完整實作，零複製回傳子切片 | ✅ |
| 拒絕空 prefix | `prefix.is_empty()` 檢查 | ✅ |
| 拒絕 prefix > 256 Unicode scalars | `prefix.chars().count() > 256` | ✅ |
| 拒絕 prefix 不符（byte-exact）| `!candidate.starts_with(prefix)` | ✅ |
| 拒絕空 suffix | `suffix.is_empty()` | ✅ |
| 拒絕純空白 suffix | `suffix.chars().all(char::is_whitespace)` | ✅ |
| 拒絕 ASCII 控制字元（<0x20, 0x7F）| `(ch as u32) < 0x20 \|\| ch == '\x7F'` | ✅ |
| 拒絕 U+2028 / U+2029 | 明確比對 `LINE_SEP`, `PARA_SEP` | ✅ |
| 英文：僅 ASCII 字母與普通空格 | `c.is_ascii_alphabetic() \|\| c == ' '` | ✅ |
| 英文：≤ 32 characters | `suffix.len() > 32` | ✅ |
| 英文：≤ 4 words | `word_count > 4` | ✅ |
| 繁中：≤ 8 Unicode scalars | `suffix.chars().count() > 8` | ✅ |
| 不 trim / normalize / 改大小寫 | 整條路徑無任何此類操作 | ✅ |

---

## 測試結果

**命令：**
```powershell
$env:PATH += ';' + $env:USERPROFILE + '\.cargo\bin'
$env:CARGO_TARGET_DIR = Join-Path $env:TEMP 'ckai-agy-ollama-target'
cargo test --manifest-path policy-core/Cargo.toml --locked
```

**結果：**
```
test result: ok. 70 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**退出碼：** 0

### prediction 模組新增測試（29 個）

| 測試名稱 | 分類 | 結果 |
|----------|------|------|
| `accepts_valid_chinese_continuation` | 基本接受 | PASS |
| `accepts_valid_english_continuation` | 基本接受 | PASS |
| `accepts_english_suffix_at_exact_32_chars` | 邊界 | PASS |
| `accepts_english_exactly_four_words` | 邊界 | PASS |
| `accepts_chinese_suffix_at_exactly_8_scalars` | 邊界 | PASS |
| `accepts_prefix_at_exactly_256_scalars` | 邊界 | PASS |
| `rejects_empty_prefix` | Prefix 拒絕 | PASS |
| `rejects_prefix_over_256_scalars` | Prefix 拒絕 | PASS |
| `rejects_prefix_mismatch_different_case` | Prefix 拒絕 / 大小寫 | PASS |
| `rejects_prefix_mismatch_traditional_vs_simplified` | Prefix 拒絕 / Unicode | PASS |
| `rejects_prefix_mismatch_decomposed_vs_composed` | Prefix 拒絕 / decomposed | PASS |
| `rejects_prefix_mismatch_whitespace_changed` | Prefix 拒絕 / 空白 | PASS |
| `rejects_empty_suffix` | Suffix 拒絕 | PASS |
| `rejects_whitespace_only_suffix` | Suffix 拒絕 | PASS |
| `rejects_suffix_with_ascii_control` | 控制字元 | PASS |
| `rejects_suffix_with_unicode_line_separator` | U+2028 | PASS |
| `rejects_suffix_with_unicode_paragraph_separator` | U+2029 | PASS |
| `rejects_english_suffix_with_digit` | 英文規則 | PASS |
| `rejects_english_suffix_with_non_ascii_letter` | 英文規則 | PASS |
| `rejects_english_suffix_exceeding_32_chars` | 英文規則 | PASS |
| `rejects_english_suffix_exceeding_4_words` | 英文規則 | PASS |
| `rejects_english_no_actual_words` | 英文規則 | PASS |
| `rejects_chinese_suffix_exceeding_8_scalars` | 繁中規則 | PASS |
| `accepts_chinese_suffix_with_mixed_punctuation` | 繁中規則 | PASS |
| `rejects_chinese_suffix_containing_newline` | 繁中規則 | PASS |
| `prefix_with_multibyte_chars_matched_exactly` | Multi-byte UTF-8 | PASS |
| `rejects_candidate_that_only_shares_ascii_prefix_bytes_not_unicode_boundary` | UTF-8 邊界 | PASS |
| `rejects_case_changed_prefix` | 大小寫保護 | PASS |
| `accepts_english_suffix_mixed_case_in_valid_range` | 大小寫 | PASS |
| `single_scalar_prefix_and_single_scalar_suffix_chinese` | 邊緣 | PASS |
| `single_ascii_prefix_and_single_ascii_suffix_english` | 邊緣 | PASS |
| `chinese_suffix_with_fullwidth_space_counts_as_one_scalar` | 全形空白 | PASS |
| `chinese_accepts_fullwidth_space_followed_by_text` | 全形空白 | PASS |

---

## Commit 資訊

```
Commit:    74dc146f622a322f24361c4f7eefea5b54c8beb2
Author:    AGY <ckai-dev@local>
Committer: AGY <ckai-dev@local>
Date:      Fri Nov 6 01:22:56 2026 +0800
Branch:    feature/ollama-phrase-prediction
Signed:    No (--no-gpg-sign)
```

**變更檔案（2 files, +435 lines）：**
- `policy-core/src/prediction.rs` *(new, +433)*
- `policy-core/src/lib.rs` *(modified, +2)*

**未觸碰：**
- `docs/verification/runs/RUN-20261105-OLLAMA-AGY-01.md` (保留舊報告)
- `prediction_session.rs` (Codex 負責，未建立)
- 所有巢狀儲存庫（`upstream/windows-chewing-tsf`, `upstream/windows-chewing-preferences`）
- `main` 分支
- 任何安裝或使用者個人詞庫

---

## 尚未串接之項目（Remaining Gaps）

| 項目 | 狀態 | 說明 |
|------|------|------|
| `prediction_session.rs` | BLOCKED (Codex 負責) | Codex 已完成 lifecycle 模組，待整合 |
| Ollama HTTP transport / `OllamaClient` | NOT_RUN | 無本機 Ollama (port 11434 未開) |
| TSF 非同步整合 (tip crate) | NOT_RUN | 第二檢查點工作 |
| ChewingPreferences 設定頁 Ollama 分頁 | NOT_RUN | 第二檢查點工作 |
| 實鍵驗收清單 | NOT_RUN | 需 GUI 與實體鍵盤 |
| 本機模型合成案例測試 | BLOCKED_MODEL | 無已安裝模型 |
| 假伺服器契約測試 | NOT_RUN | 第一檢查點後續工作 |
