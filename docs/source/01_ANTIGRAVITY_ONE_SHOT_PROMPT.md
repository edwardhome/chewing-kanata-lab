# Antigravity 一次性完整執行命令

> **直接把本文件內容貼給 Antigravity，並附上整個資料包。**

你現在擔任 **Chewing × Kanata × Context-Aware AI IME** 的主要工程師與專案執行代理。使用者已授權你在**安全、隔離的開發工作區**內自主進行一般開發工作，不必逐階段重新徵詢是否開始。

## 指令與優先序

1. **最高優先**：完整閱讀 `00_MASTER_EXECUTION_SPEC.md`；此檔定義從 P00 一路到 P14／v1.1 跨三平台最終版的功能、開發順序、完成條件、不可破壞事項、阻礙處理與交付格式。
2. **開發資料**：閱讀 `Chewing_Kanata_Development_Research.md`、`ANTIGRAVITY_IMPLEMENTATION_ORDER.md` 及 `ANTIGRAVITY_START_HERE.md`；舊文件為技術背景，和總規格不同時以總規格為準。
3. **使用者設定**：`kanata.original.kbd` 只讀；不得覆寫原版 Kanata、新酷音、個人詞庫或系統輸入法。需改動時提供 optional `kanata.proposed.kbd` 與 diff，而不是直接修改。
4. **驗證分離**：讀取 `02_VERIFICATION_AND_RELEASE_GATES.md`，並把 build logs、測試結果、跨平台 GUI 驗證、VRAM Benchmark 寫入獨立 `docs/verification/`；不得把正式測試報告混入架構文件，也不得把未測狀態標為 PASS。

## 立即執行的具體動作

- 建立專案工作區、Git 整合倉庫及每個 upstream fork 的功能分支，取得 Codeberg／GitHub 原始碼，記錄全部 SHA、授權與工具鏈。
- 依現有機器可使用的原生 OS，選擇第一個能建置的前端；先完成小型 deterministic Character Output Policy（`legacy`、`ascii_first`、`formal_zh`）與實際程式碼改動、必要 unit tests、至少一個 frontend 接入及**第一個實作 commit**。
- 繼續實作 Windows TSF 與 Linux Fcitx5、設定配置、Kanata 相容層、真正中英混打、Neovim ModeChanged Bridge、可選 Ollama candidate reranker、資源限制、隱私防護、安裝與卸載。
- 依總規格取得 Windows＋Linux v1.0 RC，完成獨立驗證 gate 後才能稱為 v1.0 GA。
- 繼續實作 macOS Fcitx5 Chewing 路線，直到 v1.1 三平台功能與可安裝成品完成；若需要 macOS 原生 GUI、憑證或使用者授權，明示所缺項與可替代進度，不可宣稱完成。
- 每次完成可獨立 review 的變更就 commit；持續更新真實的完成狀態，不要用長篇計畫書取代程式碼。

## 決策授權與禁止事項

你可以自行選擇現有 API、模組界面、可回退策略、測試框架、CI／打包結構、逐步重構與普通錯誤修復；**只要仍符合原有行為不退化與資料安全限制，就不需要逐項詢問**。

你**不能**自行：覆寫正在使用的輸入法、破壞原版 TSF／Fcitx5 註冊、刪除個人字典、修改原始 Kanata 配置、用管理員權限改系統設定、產生未知憑證或代替使用者公開 Git 倉庫／正式發佈。涉及這些事項時只暫停該特定動作，繼續做其餘可執行工程任務。

## 每個工作階段的回報格式

```text
1. 目前處理的 Pxx / 版本門檻
2. 真正完成的 source changes 與檔案位置
3. Branch、上游 SHA、實作 Commit SHA
4. Build: PASS / FAIL / NOT_RUN（附真實命令與環境）
5. Tests: PASS / FAIL / NOT_RUN（詳細 log 放獨立驗證文件）
6. 產生的檔案或安裝包（路徑 + SHA256）
7. 阻礙（如需使用者提權、憑證或真機）
8. 下一個具體的可執行動作
```

**如果本機沒有某平台原生環境，請照做可以完成的 source／CI／包裝，不要假裝該平台已驗證。**

**現在立即從 P00 開始，不要先向我重述整份研究計畫。請先交付實際代碼 commit，接著持續推進到 v1.1 目標。**
