# Windows 共用開發目錄

更新時間：2026-11-06T18:03:00+08:00

使用者指定的唯一 Windows 工作目錄為 `C:\Users\Defu\Documents\gis-code\chewing-kanata-lab`。Mac 儲存庫可作來源審查、提交同步及保存交付檔；Windows 修改與後續建置以這個共用目錄為準。

| 儲存庫 | Windows 工作分支 | 1.1.0 起點／正式版來源 |
| --- | --- | --- |
| 外層 chewing-kanata-lab | feature/1.1.0 | 已合併 1.0.2 main，另含 V110_AGY_IMPLEMENTATION.md 任務 |
| upstream/windows-chewing-tsf | feature/1.1.0 | f342e38e711442d5768593fd287d38ed98215ec8；AGY A 階段開發中 |
| upstream/windows-chewing-preferences | feature/1.1.0 | a9b224315486caa6dcd1b3f002bd2f905686b771；等待 C 階段 |


本輪 Codex 負責程式審查、設定與詞庫整合；AGY 負責傳輸模組及打包整理。Zbook 由使用者自行安裝、移除及操作驗收。共用資料夾不是兩個代理可以同時切換分支或修改同一檔案的授權；領取下一項工作前先協調檔案範圍。未完成的 AI 實驗不得直接覆蓋已驗收版本。

Agy 原有四個未提交檔案已保存於原生分支 `research/agy-ai-before-settings-20261105`，commit `e66ef6c`。完整歷史、diff、engineering-terms.json 與檔案雜湊備份位於 `C:\Users\Defu\Documents\AI-Chewing\shared-workspace-backup-20261105T185040`。需要接續時先審查差異，再選擇性整合；不要直接切回該分支造成版本倒退。

原先 `ckai-codex` 與 `ckai-package-source` 保留為歷史建置／來源參考，不刪除。未合併實驗及私人詞庫均未發布。

1.0.1 曾獲使用者確認完整設定頁與選字正常；本輪使用者確認 preview.2 目前功能正常，已授權發布 1.0.2，Zbook 仍由使用者自行安裝。F 延遲在完整設定頁的「其他設定」。右下角「設定」由當前載入的輸入法 DLL 開啟同版本 EXE；Explorer 若仍載入舊 DLL，應由使用者儲存工作後登出登入，避免直接覆寫仍使用中的舊套件。

## 第一批歷史：Ollama 詞組預測

使用者指定 AGY 與 Codex 分工實作、以 Git 協作；詳見 [開發任務](../tasks/OLLAMA_PHRASE_PREDICTION_AGY.md) 及 [第一批分工](../tasks/OLLAMA_PARALLEL_CHECKPOINT_01.md)。AGY 負責續寫驗證，Codex 負責請求生命週期及整合；不在相同檔案平行修改或切換分支。2026-11-06T01:32:06+08:00，整合程式版本 `1dcd371` 已同步到 Windows 共用分支，93 個核心測試通過。後續 HTTP／TSF／設定頁仍待實作，已安裝的 1.0.1 保持不變。

## 目前工作：1.0.2 第二批

AGY 的傳輸提交 `6746219`、`cd63e83` 已保留作者歷史整合；完整設定頁新增自訂工程詞彙與 GitHub 手動檢查／下載。外層與 nested preferences 已以 fast-forward 同步至共用目錄，原生 TSF 分支及既有未追蹤報告保留。詳細測試、打包與未完成項目見 [第二批紀錄](../verification/runs/RUN-20261106-102-SETTINGS-TRANSPORT.md)。

## 1.0.2-preview.2：中文恢復

已驗收來源包含 AGY 的 RecoveryGate 與 Codex 的原生／設定整合。原生與設定完整 patch tree 均已對齊；49 項本輪測試與兩者 Release 建置通過。AGY 後續 503 導致封裝改由 Codex 完成，未安裝到使用者端；詳見 [恢復驗證](../verification/runs/RUN-20261106-INPUT-RECOVERY.md)。

下一輪先確認共用 Git 狀態及既有 AGY 程序／對話；本輪主對話 `9718153a-085c-4fb6-bfa4-219555c0aaac` 保留。切勿把尚未完成的 AGY 報告當成新增程式碼或自動合併其後續改動。

## 1.1.0 交付與目前程序

更新時間：2026-11-07T17:46:34+08:00

共用 Windows 來源已對齊外層 `8c03249`、原生 `3279313`、設定 `914f05f`，均在 feature/1.1.0。AGY 的 A 階段及防護提交保留作者歷史；Codex 完成佇列生命週期、共用候選、中文別名／設定及驗收修正。上方起點表與舊版程序名稱屬歷史資料。

AGY 保持對話 `9718153a-085c-4fb6-bfa4-219555c0aaac`；目前 Codex 已要求停止程式修改，改作唯讀建置需求查核。接手前仍須核對實際程序與工作樹，不可依歷史 Remote Control 名稱判定程序存活。

本版隔離建置來源 `%TEMP%\ckai-catalog-110-review`、target `%TEMP%\ckai-catalog-110-target`；封裝來源 worktree `%TEMP%\ckai-package-final-110` 固定外層提交。196 個建置來源／dist 雜湊與固定來源一致，舊 1.0.2 成品及 target 保留。

詳見 [1.1.0 整合驗證](../verification/runs/RUN-20261107-110-INTEGRATION.md)。使用者自行安裝及實體按鍵驗收；沒有遠端安裝 Zbook。
