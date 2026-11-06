# 開發紀錄與變更日誌 (Development Changelog)

> **專案**：新酷音 × Kanata 跨平台輸入法 (Chewing + Kanata Lab)  
> **當前開發分支**：`feature/ollama-lifecycle`

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
