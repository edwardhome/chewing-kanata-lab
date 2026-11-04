# AI-Bopomofo 功能對齊矩陣

> 參考版本：AI-Bopomofo 0.6.14 commit `9700690321df88`  
> 目標平台：Windows TSF (chewing_tip DLL)  
> 更新日期：2026-11-05 (feature/ai-bopomofo-parity Stage A)

**狀態說明**

| 標記 | 說明 |
|------|------|
| ✅ INTEGRATED | 已整合進 TSF DLL，有自動化測試 |
| 🔵 IMPLEMENTED | 邏輯實作完成，待 TSF 整合 |
| 🟡 PARTIAL | 部分實作，有已知差距 |
| ⬜ NOT_STARTED | 尚未開始 |
| 🔴 BLOCKED | 有已知阻礙 |
| 🚫 NOT_RUN | GUI/鍵盤手動測試，未實際執行 |

---

## Stage A — 來源交付差距

| 項目 | 狀態 | 說明 |
|------|------|------|
| patch series 匯出 (6 commits) | ✅ INTEGRATED | `docs/patches-windows-tsf/0001-0006.patch` |
| apply/bootstrap 腳本 | ✅ INTEGRATED | `scripts/apply_patches.ps1` |
| 建置指南 | ✅ INTEGRATED | `docs/development/BOOTSTRAP_WINDOWS.md` |
| SHA 驗證腳本 | ✅ INTEGRATED | `scripts/apply_patches.ps1 -Check` |
| adapter_tsf.patch 錯誤修正 | ✅ INTEGRATED | 文件化於 BOOTSTRAP_WINDOWS.md §3 |
| adapter_fcitx5.patch 錯誤修正 | 🟡 PARTIAL | 錯誤已文件化，C API 需重寫才可用 |
| Linux Fcitx5 支援 | 🔴 BLOCKED | 標記 STALE，C API 需重寫 |
| vendor-manifest.lock / Cargo.lock 追蹤 | ✅ INTEGRATED | Cargo.lock 在 upstream git history |

---

## Stage B — Windows 鍵盤行為對齊

### B1. F 分層 (QuickSymbolLayer / FHoldGesture)

| 功能 | 參考行為 | Windows 狀態 | 測試 |
|------|---------|------------|------|
| F 短按 → 保留原注音 ㄑ | 不進入分層，回傳 ㄑ | ✅ INTEGRATED | ✅ 自動化 |
| F 短按英文模式 → f/F | 直接輸出 f/F | ✅ INTEGRATED | 🚫 NOT_RUN |
| F 長按 ≥150ms → 進入分層 | `FHoldGesture.phase == .held` | ✅ INTEGRATED | ✅ 自動化 |
| 分層內 Esc → 輸出反引號 | `FHoldGesture`: Esc → `` ` `` | ✅ INTEGRATED | ✅ 自動化 |
| 分層內 Backspace → 刪除組字 | 刪除組字緩衝，不越界 | 🟡 PARTIAL | ⬜ 缺測試 |
| 連續 Backspace 刪除 | 多次退格 | 🟡 PARTIAL | ⬜ 缺測試 |
| 放開 F → 送出組字 | `endLayer` → commit | ✅ INTEGRATED | 🚫 NOT_RUN |
| F+Shift → 大寫 Shift 入口 | Shift+F 短按保留大寫入口 | 🟡 PARTIAL | ⬜ 缺測試 |
| 自動重複不進入分層 | `repeating: true` → 阻止 | ✅ INTEGRATED | ✅ 自動化 |
| 修飾鍵不套用符號映射 | Ctrl/Alt/Super → passthrough | ✅ INTEGRATED | ✅ 自動化 |
| F+U/M 在門檻內 → 注音優先 | `prefersPhonetic` ㄑㄧ/ㄑㄩ | ✅ INTEGRATED | ✅ 自動化 |
| F+U/M 超過門檻 → 數字 8/2 | 長按後正常分層 | ✅ INTEGRATED | ✅ 自動化 |
| Shift+frrrr → + 符號序列 | Shift+F+R → `+` | ✅ INTEGRATED | ⬜ 缺整合測試 |
| 密碼欄/Secure Input 停用 | `SecureInput` → 不啟用 | ✅ INTEGRATED | 🚫 NOT_RUN |
| `FHoldGesture` 狀態機用 `FHoldEvent`/`Effect` | policy-core pub API | 🟡 PARTIAL | — |
| chewing.rs 使用 policy-core FHoldGesture | 目前用自建 FKeyState | 🟡 PARTIAL | — |
| 偏好設定門檻 100-1000ms, 預設 150ms | `f_hold_delay_ms` | ✅ INTEGRATED | 🚫 NOT_RUN |
| `SymbolKeyOwnership` 重複事件管理 | 分層後放開 F 仍抑制 repeat | ⬜ NOT_STARTED | ⬜ |

### B2. 雙擊 Shift 切換 (DoubleShiftGesture)

| 功能 | 參考行為 | Windows 狀態 | 測試 |
|------|---------|------------|------|
| 雙擊 Shift 切換直接英文 | `DoubleShiftGesture` 250ms tap, 350ms interval | ✅ INTEGRATED | ✅ 自動化 |
| 打字/其他修飾鍵取消手勢 | `reset()` on typing | ✅ INTEGRATED | ✅ 自動化 |
| 長按不觸發 | `tapLimit = 0.25s` | ✅ INTEGRATED | ✅ 自動化 |
| 組字中不觸發 | `chewing_editor.is_composing()` | ✅ INTEGRATED | 🚫 NOT_RUN |
| Focus 切換 reset | `on_deactivate` reset | ✅ INTEGRATED | 🚫 NOT_RUN |

### B3. Shift 英文連續輸入 (ShiftLetterSession)

| 功能 | 參考行為 | Windows 狀態 | 測試 |
|------|---------|------------|------|
| Shift + 字母開啟英文組字 | `ShiftLetterSession::start_with_letter` | ✅ INTEGRATED | 🟡 部分測試 |
| 繼續 Shift 大寫輸入 | 持續 Shift 維持大寫 | ✅ INTEGRATED | 🚫 NOT_RUN |
| 刪至空白繼續 continuation | 刪除至空組字仍留 session | 🟡 PARTIAL | ⬜ 缺測試 |
| 游標編輯 (Left/Right/Home/End) | 移動游標 | 🟡 PARTIAL | ⬜ 缺測試 |
| 選字後 Esc 還原原始字串 | Esc → 還原 original | 🟡 PARTIAL | ⬜ 缺測試 |
| 明確 Shift 英文單字模式 | 選單入口切換 | ✅ INTEGRATED | 🚫 NOT_RUN |
| 原始輸入字串第一候選 | original first | ✅ INTEGRATED | ✅ 自動化 |
| Tab/Down 循環候選 | `CompositionAction::Cycle` | ✅ INTEGRATED | ✅ 自動化 |
| Enter 確認 | commit | ✅ INTEGRATED | 🚫 NOT_RUN |
| Space 確認 + 加空格 | commit + space | ✅ INTEGRATED | 🚫 NOT_RUN |
| 阿拉伯數字直接輸入 | digits passthrough | ✅ INTEGRATED | 🚫 NOT_RUN |
| 持續 Shift 大寫不觸發 AI | `suppressAI` on sustained shift | ⬜ NOT_STARTED | ⬜ |
| 直接英文穩定行為 | English mode stable | ✅ INTEGRATED | 🚫 NOT_RUN |
| 標準注音傳統輸入保留 | upstream 傳統輸入不影響 | ✅ INTEGRATED | 🚫 NOT_RUN |
| 半形 Markdown 標點 | 預設半形 | ✅ INTEGRATED | 🚫 NOT_RUN |

### B4. ASCII 組字 / 中英混打

| 功能 | 參考行為 | Windows 狀態 | 測試 |
|------|---------|------------|------|
| 注音音節保留 (active syllable) | F tap → 先回 ㄑ 再處理 | ✅ INTEGRATED | ✅ 自動化 |
| 圍繞既有 ASCII 組字恢復 | F release → restore | ✅ INTEGRATED | 🚫 NOT_RUN |
| 數字鍵保留注音選字功能 | 中文模式數字選字 | ✅ INTEGRATED | 🚫 NOT_RUN |

---

## Stage C — 工程/科學詞庫

| 項目 | 狀態 | 說明 |
|------|------|------|
| 103 個參考詞條載入 | ✅ INTEGRATED | `policy-core` `TechnicalLexicon` |
| 詞條 JSON schema 驗證 | ✅ INTEGRATED | `EngineeringLexicon` schema checks |
| 兩字元精確比對 | ✅ INTEGRATED | `exact_index` |
| 三字元以上前綴展開 | ✅ INTEGRATED | `prefix_index` |
| 原始輸入第一候選 | ✅ INTEGRATED | `candidateItems` original first |
| 最多八個額外候選 | ✅ INTEGRATED | `.prefix(8)` |
| 明確選擇才送出 | ✅ INTEGRATED | 無自動補全 |
| Unicode/ASCII 變體 | ✅ INTEGRATED | `outputs` 陣列 |
| 現有/私有/已核准候選優先 | ✅ INTEGRATED | `approved` 優先 |
| 私有詞庫絕不讀取/覆寫 | ✅ INTEGRATED | AGENTS.md 合規 |
| TSF 整合（候選視窗顯示） | 🟡 PARTIAL | 需接入 candidate_list |
| Ollama 建議低於詞庫優先序 | ⬜ NOT_STARTED | 見 Stage D |
| 別名大小寫不敏感 | ✅ INTEGRATED | `.lowercased()` 比對 |

---

## Stage D — 本機 Ollama 整合

| 項目 | 狀態 | 說明 |
|------|------|------|
| loopback Ollama 後端 (127.0.0.1:11434) | ✅ INTEGRATED | `ai_client.rs` |
| 無同步 HTTP 阻塞 | ✅ INTEGRATED | 背景 worker thread |
| 一個推論槽，請求取消 | 🟡 PARTIAL | revision guard 存在，需驗證 |
| 回應限制與超時 | ✅ INTEGRATED | 80ms connect, 200ms read |
| 組字/焦點/模式版本驗證 | 🟡 PARTIAL | revision guard 存在 |
| 過期回應丟棄 | ✅ INTEGRATED | `revision` check |
| 沙盒環境停用 | ✅ INTEGRATED | `is_sandboxed_or_restricted_process()` |
| **現有 AiRerankerClient 功能** | 候選字語意重排 | ✅ INTEGRATED |
| **缺：組字糾正** | 2+ 漢字完整組字 → 糾正 | ⬜ NOT_STARTED |
| **缺：英文前綴保留預測** | 3-64 字元 token → 預測 | ⬜ NOT_STARTED |
| **缺：自動輔助去抖動 700ms** | debounce before request | ⬜ NOT_STARTED |
| **缺：最短 2 秒請求間隔** | rate limit | ⬜ NOT_STARTED |
| **缺：每次同一組字一次嘗試** | per-composition attempt limit | ⬜ NOT_STARTED |
| **缺：自動輔助預設開啟/獨立切換** | toggle | ⬜ NOT_STARTED |
| **缺：Tab 選擇 AI 建議** | explicit Tab → Enter/click | ⬜ NOT_STARTED |
| **缺：Esc 隱藏但不丟失輸入** | Esc → hide suggestions | ⬜ NOT_STARTED |
| **缺：持續大寫抑制** | sustained shift → suppress | ⬜ NOT_STARTED |
| **缺：讀取已安裝本地模型** | list models without download | ⬜ NOT_STARTED |
| **缺：可設定模型選擇** | model config | ⬜ NOT_STARTED |
| Ollama 中斷不影響打字 | silent fallback | ✅ INTEGRATED |
| 無 `fm` 依賴 | Apple Foundation Models 不支援 | ✅ INTEGRATED |
| 密碼欄停用 | secure_field | ✅ INTEGRATED (guard exists) |
| 漢字數量不變驗證 | Han count invariant | ✅ INTEGRATED (`policy-core`) |
| 技術 Unicode 身份驗證 | Unicode identity | ✅ INTEGRATED (`policy-core`) |

---

## Stage E — 學習詞庫

| 項目 | 狀態 | 說明 |
|------|------|------|
| ChewingDev 命名空間隔離 | ✅ INTEGRATED | `%APPDATA%\ChewingDev` |
| 有界非同步觀察 | ⬜ NOT_STARTED | LexiconLearning 移植 |
| 僅可靠已選詞條觀察 | ⬜ NOT_STARTED | `candidate_only` first |
| 無完整句子/原始打字歷史 | ⬜ NOT_STARTED | 設計規範 |
| JSON 匯出/匯入 schema 限制 | ⬜ NOT_STARTED | 移植 LexiconSchema.swift |
| 明確本地核准 | ⬜ NOT_STARTED | approval gate |
| 停用/回滾 | ⬜ NOT_STARTED | 設計 |
| 不可變版本快照 | ⬜ NOT_STARTED | 設計 |
| SQLite 失敗不阻塞輸入 | ⬜ NOT_STARTED | 錯誤隔離 |
| 閒置 Ollama 組織（限核准範圍） | ⬜ NOT_STARTED | idle organizer |
| 觀察/組織/部署獨立開關 | ⬜ NOT_STARTED | 設計 |
| 不修改 upstream/使用者詞庫 | ✅ INTEGRATED | AGENTS.md 合規 |

---

## 自動化測試計數

> 來源 SHA: `2232ff9b91efae8247842eb0d34b8880c30d8e50`  
> 指令: `cargo test --workspace --lib` (windows-chewing-tsf)  
> 指令: `cargo test --lib` (policy-core)

| 專案 | 測試數 | 通過 | 失敗 | 跳過 |
|------|--------|------|------|------|
| `chewing_tip` | 13 | 13 | 0 | 0 |
| `chewing_tip_core` | 8 | 8 | 0 | 0 |
| `chewing_policy_core` | 25 | 25 | 0 | 0 |
| **合計** | **46** | **46** | **0** | **0** |

---

## 主要待完成差距

依優先序：

1. **Stage D — AI 組字糾正與英文預測**（全新功能，非僅重排）
2. **Stage B — `SymbolKeyOwnership` 整合**（分層內重複事件）
3. **Stage B — ShiftLetterSession 完整游標/Esc 語義**
4. **Stage C — 詞庫候選視窗 TSF 整合**（需接入 candidate_list）
5. **Stage E — LexiconLearning 移植**（全新）

---

_最後更新: 2026-11-05 Stage A 完成後_
