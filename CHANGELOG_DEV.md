# 開發紀錄與變更日誌 (Development Changelog)

> **專案**：新酷音 × Kanata 跨平台輸入法 (Chewing + Kanata Lab)  
> **當前開發分支**：`release/v1.1.1`

---

## [1.1.1] 共用個人詞庫

- 新增 ai-bopomofo.lexicon 1.0.0 交換檔的匯入預覽、本機確認、獨立儲存、校驗及前版回復。
- 共用中文候選接入原生選單；工程詞可由使用者選取後匯出；保留原始輸入與明確選取。
- 中文別名不在 v1 格式內，預設拒絕有損匯出；明確允許省略時列出清單且保留原檔。
- 不含私人中文詞庫自動匯出，也未啟用 Ollama 背景輸入蒐集。
- Windows 270 項通過、1 項略過；GUI／實體按鍵驗收仍為 NOT_RUN。

---

## [1.0.2] 正式版

- 使用者確認 preview.2 目前功能正常可用，授權轉為正式版本。
- 保留工程詞彙編輯、批次文本與大模型提示詞、GitHub 手動檢查更新，以及 Restart 恢復中文輸入。
- 正式版識別同步至 Windows 原生資源、Cargo 與設定程式，未混入 1.1.0 的行為修改。
- 已知新增需求：注音組字中的 F 分層、Shift 英文交接與共用工程候選，排入 1.1.0；未宣稱本版已修正。

---

## [1.0.2-preview.2] 開發中

- 新增原生右下角選單與完整設定頁「其他設定」中的 `Restart：恢復中文輸入`。
- 恢復要求有效 30 秒；保留未送出組字、候選及 F／修飾鍵手勢，CapsLock 英文鎖定時需先切回中文狀態。
- 修正漏掉 Shift 放開事件時的殘留判斷；設定或焦點更新不再重建仍有組字的 editor，重建後同步語言與全半形模式。
- AGY 實作恢復要求判斷與測試，Codex 完成原生／設定整合及時間校正、可重入通知邊界修正。
- 本輪 Windows 原生 40 項、設定 9 項測試通過，兩者 Release 與前端建置通過。未安裝、未重啟使用者輸入法，Zbook 實鍵待驗收。
- [恢復功能與驗證](docs/verification/runs/RUN-20261106-INPUT-RECOVERY.md)。

---

## [1.0.2-preview.1] 開發中

- 新增自訂工程詞彙設定表單、個人輸入碼映射、批次文本匯入預覽與提示詞匯出；內建補上 H2O → H₂O。
- 文本附繁體中文說明、TSV 範例與可貼給外部模型的整理提示詞。儲存後需重開使用輸入法的應用程式。
- 設定內新增 GitHub 手動檢查更新／下載 Windows x64 套件；使用者自行安裝，不自動啟動安裝程式。
- 與 AGY 共同完成 Ollama 生命週期、JSON 驗證與有期限的本機 HTTP 傳輸。尚未接入 TSF 候選，AI 不因本批而啟用。
- 165 項無外網測試與 1 項 GitHub 實際查詢通過；前端、原生輸入法與設定程式 Release 建置通過。GUI／Zbook 實鍵驗收待使用者操作。
- [驗證紀錄](docs/verification/runs/RUN-20261106-102-SETTINGS-TRANSPORT.md)；[第二批分工](docs/tasks/OLLAMA_PARALLEL_CHECKPOINT_02.md)。

---

## [Phase 1 - Sprint A & B] 2026-10-22

### 1. 實際完成項目 (Completed)
- **Windows TSF 原生 Release 二進位建置完成**：
  - 配置 Rust 1.99.1 + Visual Studio 2026 MSVC (19.50) 工具鏈。
  - 編譯產出 `dist/windows/chewing_tip.dll` (4.1MB, SHA256: `F8ACC7FF...`)。
  - 編譯產出 `dist/windows/tsfreg.exe` (221KB, SHA256: `520C7039...`)。
  - 成功執行本地隔離註冊 (`packaging/windows/register_dev_tsf.ps1`)，註冊於 HKCU 獨立 GUID。
  - 自動部署設定檔至 `%APPDATA%\ChewingDev\policy.toml`。
- **分支建立**：由 `research/upstream-audit` 建立獨立功能分支 `feature/chewing-ascii-first-mvp`。
- **上游工作樹鎖定**：
  - `libchewing`: `ec7f595ea7726fa06a9493e819b81669eba605a5` (`0.14.0-alpha.4`)
  - `windows-chewing-tsf`: `053d31ba10c089ae98508dd4a908ac2c2be3cb49` (`26.9.0`)
  - `fcitx5-chewing`: `60096e8ef49df21de6c155d7a0785a58fcccb955` (`5.1.13`)
  - `ibus-chewing`: `6e885f1a8556cb8cfcc657c5b0cfd71b93db4c5c` (`2.2.0`)
  - `kanata`: 原始配置基準 `config/kanata.original.kbd` 嚴格唯讀。
- **實作 `policy-core` 策略模組**：
  - 建立三種核心模式：`legacy` (預設回退)、`ascii_first` (語法標點半形優先)、`formal_zh` (繁體中文公文全形標點)。
  - 實作按鍵放行判定 (`policy_should_bypass_key`) 與提交字串轉換 (`policy_transform_commit`)。
  - 同時提供純 C99 實作與 Rust 介面，具備零外部依賴、純記憶體、無鎖微秒級特性。
- **完成自動化單元測試**：
  - 撰寫 `policy-core/tests/test_policy.c`，使用 GCC 6.3 於本機編譯通過，**32 / 32 項斷言全部 PASS**。
  - 撰寫 `adapters/fcitx5/tests/test_fcitx5_adapter.cpp`，使用 G++ 6.3 編譯通過，**5 / 5 項斷言全部 PASS**。
- **建立雙平台真實 Adapter 補丁草案**：
  - Windows TSF: `adapters/windows-tsf/adapter_tsf.patch`
  - Linux Fcitx5: `adapters/fcitx5/adapter_fcitx5.patch`
- **隔離防護**：
  - 定義獨立 GUID 與 `%APPDATA%\ChewingDev` 目錄，保證開發過程中不覆蓋系統正式新酷音與個人詞庫。

---

### 2. 已知邊界與限制 (Known Boundaries)
- **Kanata Unicode 注入**：Kanata 的 `(unicode u+003a)` 動作走 OS 注入路徑，若文字已被目標應用程式直接接收並經應用程式內部「智慧標點」後處理轉為全形，輸入法無權亦無法反向攔截。
- **Windows Rust 建置條件**：目前開發主機已具備 Visual Studio 18 MSVC 建置環境與 MinGW GCC/G++，但尚未配置全域 Rust `cargo`，Windows TSF 之完整安裝器建置需依賴本地 Rust 工具鏈就緒。

---

### 3. 一鍵回退路線 (Rollback Strategy)
若使用者在日常使用中發現任何不符預期之行為，可透過以下方式立即回退：
1. **設定檔回退**：將 `policy.toml` 中之 `default_profile` 設為 `"legacy"`，所有 Bypass 與 Commit 轉換立即旁路失效，行為 100% 還原為上游原版新酷音。
2. **Kanata 零變更**：本專案完全未更動使用者現有之 `kanata.original.kbd`，使用者肌肉記憶（Caps/Ctrl/Win+Space/Number layer）不受任何破壞。

---

### 4. 待辦事項 (Backlog)
- [ ] Sprint C: 於具備 Linux 環境中編譯 Fcitx5 動態外掛並進行 X11/Wayland 桌面整合。
- [ ] Sprint D: 實作 Neovim `ModeChanged` 通訊 Bridge 測試雛型。
- [ ] Sprint E: 驗證 Ollama 非同步候選字重排序 (Reranker) 記憶體與延遲表現。
