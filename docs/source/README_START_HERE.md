# Chewing × Kanata × Context-Aware AI IME — 完整執行資料包

這是供 Antigravity 從原始碼開發直到正式發佈使用的資料包，建立日期 2026-10-22。

## 怎麼用

1. 將整個資料包解壓縮到 Antigravity 能存取的工作資料夾。
2. 打開 `01_ANTIGRAVITY_ONE_SHOT_PROMPT.md`，**完整貼入 Antigravity**。
3. Antigravity 必須先讀 `00_MASTER_EXECUTION_SPEC.md`，按 P00–P14 依序執行、維持真實 Git Commit，直到 Windows/Linux v1.0 與 macOS v1.1 目標。
4. `02_VERIFICATION_AND_RELEASE_GATES.md` 是另一篇**驗證計畫**，不是測試結果；所有真正實測內容在開發工作區的 `docs/verification/` 留存。
5. 早期兩份研究／任務文件供技術背景使用；有衝突時以 `00_MASTER_EXECUTION_SPEC.md` 為準。
6. `kanata.original.kbd` 是你的原始設定，只讀，不得改動或覆寫現有 Kanata。

## 檔案列表

- `00_MASTER_EXECUTION_SPEC.md` — 最終版總開發文件，包含工程順序、架構、程式切入點、跨平台、AI、打包與維運。
- `01_ANTIGRAVITY_ONE_SHOT_PROMPT.md` — 一次性開工命令。
- `02_VERIFICATION_AND_RELEASE_GATES.md` — 與開發內容分開的 QA/Release Gate 規劃。
- `Chewing_Kanata_Development_Research.md` — 前期詳細技術研究資料。
- `ANTIGRAVITY_IMPLEMENTATION_ORDER.md` — 前期 MVP 實作命令（歷史參考）。
- `ANTIGRAVITY_START_HERE.md` — 前期啟動說明（歷史參考）。
- `kanata.original.kbd` — 使用者目前的 Kanata 配置。

## 重要提醒

本資料包是**工程規格**，不是已完成的輸入法軟體。Antigravity 必須在自己的執行環境真的寫碼、編譯、Commit 與測試；沒有 OS／權限／憑證的項目不能假稱已完成。公開發佈與系統修改必須取得使用者授權。
