# F-Hold Gesture & Mac Semantics Parity Review (2026-11-16)

## 1. 概述 (Overview)

本文件依據使用者授權之 F 鍵長按修復工作（Task WINDOWS-F-PARITY），記錄對 `chewing-policy-core` 現行實作（commit `a75a7d0`）之 `FHoldGesture` 與 Mac c2fe36e（`QuickSymbolLayer.swift` / `quick-symbols.md`）語意一致性審查、獨立契約測試建立，以及對 Policy Core 與 Native TSF 間銜接點之審查報告。

---

## 2. 測試執行結果 (Test Execution Results)

- **測試指令**：
  ```powershell
  $env:CARGO_TARGET_DIR = "$env:TEMP\ckai-f-core-target"
  cargo test --locked --offline --all-features
  ```
- **測試總覽**：
  - `chewing-policy-core` 單元測試 (unittests `src/lib.rs`): **155 passed; 0 failed**
  - `f_hold_parity_contract.rs` (新增獨立回歸契約測試): **11 passed; 0 failed**
  - `app_input_rules_contract.rs`: **6 passed; 0 failed**
  - `prediction_contract.rs`: **5 passed; 0 failed**
  - `shared_engineering_exchange.rs`: **4 passed; 0 failed**
  - `shared_lexicon.rs`: **9 passed; 0 failed**
  - `unified_sqlite.rs`: **12 passed; 0 failed**
  - `unified_store.rs`: **12 passed; 0 failed**
  - **總測試通過數**: **214 passed; 0 failed; 0 ignored**

---

## 3. 回歸契約測試涵蓋範圍 (`tests/f_hold_parity_contract.rs`)

新增的獨立測試模組直接針對 `FHoldGesture` 規格進行黑箱與合約驗證，涵蓋以下關鍵要求：

1. **短按點擊週期 (Short Tap Lifecycle & ReplayTap)**：
   - 驗證初次 Press 事件發出 `Schedule` 與 `Consume`，進入 `Phase::Pending` 並標記 `owns_key_up = true`。
   - 門檻內（如 50ms）收到 `Release`，發出 `[Invalidate, ReplayTap, Consume]`，狀態回到 `Idle` 並釋放 ownership。
2. **嚴格門檻邊界比對 (`elapsed == delay` vs `elapsed > delay`)**：
   - 遍歷 100ms, 150ms, 200ms, 500ms, 1000ms 等延遲設定。
   - `elapsed < delay`：不觸發超時，保持 `Pending`。
   - `elapsed == delay`：**嚴格不觸發**，維持 `Pending`（絕不提前進層）。
   - `elapsed > delay`：觸發 `[BeginLayer]`，狀態轉為 `Phase::Held`。
3. **門檻內/等門檻遭遇其他鍵 (OtherKey: R, U, M, Esc)**：
   - 任何鍵於門檻內或恰在門檻邊界按下，立即判定為普通打字：發出 `[Invalidate, ReplayTap]`，狀態轉為 `Idle`，移除 immediate chord 提前分層行為。
   - 保持 `owns_key_up = true`，使實體 F 放開時能被安全 consume，不洩漏給外部應用。
4. **取消後重複按鍵與晚到逾時防護 (Repeats & Late Timeout Guard)**：
   - 遭遇 OtherKey 轉入普通打字後，驗證：
     - 晚到的計時器事件 (`Elapsed(300ms)`) 絕不重新啟用。
     - 晚到的 `Timeout` 絕不重新啟用。
     - F 鍵自動重複 (`repeating: true`) 保持被 `Consume`，不可再次進入 `Pending` 或 `Held`。
     - 直到真實實體 F `Release` 才釋放 `owns_key_up`。
5. **被動放開與延遲實體放開 (ObservedRelease & Delayed Release)**：
   - 驗證在 `Pending` 與 `Held` 狀態下，`ObservedRelease` 正確結算手勢並保留 `owns_key_up`，直到真實 `Release` 抵達時 consume。
6. **延遲夾取與快照預設 (Delay Clamping & Defaults)**：
   - 驗證 `DEFAULT_DELAY_MS` 為 150ms，夾取區間嚴格限制在 `100ms..=1000ms`。
7. **Shift+F 與英文組合交互**：
   - 驗證 `ShiftLetterSession::start_with_letter('F', ...)` 不會啟動 `initial_letter_key`，Shift+F 專屬於英文大小寫切換，不屬於長按分層。
   - 驗證 `FCompositionBuffer` 的退格操作具備邊界保護，退格符號層符號絕不破壞原先英文緩衝區。

---

## 4. Policy Core 程式碼審查與發現 (Code Audit Findings)

在審查 `chewing-policy-core/src/f_hold.rs` 時，現行核心邏輯整體設計乾淨且符合狀態機規範，但發現以下注意事項與潛在改進點（依作業規範僅作報告，不主動修改核心原始碼）：

1. **`Event::Timeout` 缺乏 elapsed 邊界檢查**：
   - `f_hold.rs` 第 131 行：
     ```rust
     Event::Elapsed(duration) => {
         if self.phase == Phase::Pending && duration > self.delay {
             self.handle(Event::Timeout)
         } else {
             vec![]
         }
     }
     ```
     在 `Event::Elapsed` 中有 `duration > self.delay` 的保護；但如果外部調用者直接發送 `Event::Timeout`（而非 `Event::Elapsed`），`Event::Timeout` 僅檢查 `self.phase == Phase::Pending` 即直接轉為 `Phase::Held`。
   - **建議**：Native TSF 應統一使用 `Event::Elapsed(elapsed)` 作為計時器回報，避免未核對實際經過時間即直接送出 `Event::Timeout`；或者核心可以在 `Event::Timeout` 也保留延遲驗證機制。

2. **Native TSF 銜接點之已知缺陷（與 Codex 審查意見對齊）**：
   - 審查確認先前在 native commit `8ec6227` 的實作中存在以下已知缺陷（目前由 Codex 負責原生層修正）：
     1. **注音預覽硬編碼 (Hardcoded Zhuyin)**：在 `chewing.rs` 中使用 `f_hold_zhuyin_preview` 硬編碼為 `"ㄑ"`，未根據使用者實際鍵盤配置（如大千、許氏、倚天、倚天26鍵等）動態計算對應注音符號。
     2. **既有 Preedit 被覆寫 (Loss of Existing Preedit)**：直接使用 `insert_text_at_selection` 或粗暴更換 preedit，可能破壞原先已存在之組字緩衝區。
     3. **Timer 上下文與重試機制 (Timer Context Mismatch & Early Tick)**：Windows Timer 若稍早於 deadline 觸發（例如系統排程誤差 1-2ms），若未排程 retry，會導致計時失效且未能再次喚醒進入分層。
   - **結論**：AGY 嚴格遵守任務界線，不宣稱 Native preview/context 已通過；後續由 Codex 完成原生層補正。
