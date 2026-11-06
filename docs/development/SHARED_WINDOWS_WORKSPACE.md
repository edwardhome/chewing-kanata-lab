# Windows 共用開發目錄

更新時間：2026-11-06T11:05:59+08:00

使用者指定的唯一 Windows 工作目錄為 `C:\Users\Defu\Documents\gis-code\chewing-kanata-lab`。Mac 儲存庫可作來源審查、提交同步及保存交付檔；Windows 修改與後續建置以這個共用目錄為準。

| 儲存庫 | 當前工作分支 | 已驗收的原生來源 |
| --- | --- | --- |
| 外層 chewing-kanata-lab | feature/ollama-phrase-prediction | 8af1ec4；第二批核心、傳輸及設定整合 |
| upstream/windows-chewing-tsf | feature/ollama-phrase-prediction | b5dc92e1fb8f974ed40c79b8bd31907fd2b2dd28 |
| upstream/windows-chewing-preferences | feature/ollama-phrase-prediction | 113736a891c24f1ed87f158af715f141fb7162b7 |

本輪 Codex 負責程式審查、設定與詞庫整合；AGY 負責傳輸模組及打包整理。Zbook 由使用者自行安裝、移除及操作驗收。共用資料夾不是兩個代理可以同時切換分支或修改同一檔案的授權；領取下一項工作前先協調檔案範圍。未完成的 AI 實驗不得直接覆蓋已驗收版本。

Agy 原有四個未提交檔案已保存於原生分支 `research/agy-ai-before-settings-20261105`，commit `e66ef6c`。完整歷史、diff、engineering-terms.json 與檔案雜湊備份位於 `C:\Users\Defu\Documents\AI-Chewing\shared-workspace-backup-20261105T185040`。需要接續時先審查差異，再選擇性整合；不要直接切回該分支造成版本倒退。

原先 `ckai-codex` 與 `ckai-package-source` 保留為歷史建置／來源參考，不刪除。未合併實驗及私人詞庫均未發布。

目前安裝 1.0.1，使用者確認完整設定頁與選字正常。F 延遲在完整設定頁的「其他設定」。右下角「設定」由當前載入的輸入法 DLL 開啟同版本 EXE；Explorer 若仍載入舊 DLL，應由使用者儲存工作後登出登入，避免直接覆寫仍使用中的舊套件。

## 第一批歷史：Ollama 詞組預測

使用者指定 AGY 與 Codex 分工實作、以 Git 協作；詳見 [開發任務](../tasks/OLLAMA_PHRASE_PREDICTION_AGY.md) 及 [第一批分工](../tasks/OLLAMA_PARALLEL_CHECKPOINT_01.md)。AGY 負責續寫驗證，Codex 負責請求生命週期及整合；不在相同檔案平行修改或切換分支。2026-11-06T01:32:06+08:00，整合程式版本 `1dcd371` 已同步到 Windows 共用分支，93 個核心測試通過。後續 HTTP／TSF／設定頁仍待實作，已安裝的 1.0.1 保持不變。

## 目前工作：1.0.2 第二批

AGY 的傳輸提交 `6746219`、`cd63e83` 已保留作者歷史整合；完整設定頁新增自訂工程詞彙與 GitHub 手動檢查／下載。外層與 nested preferences 已以 fast-forward 同步至共用目錄，原生 TSF 分支及既有未追蹤報告保留。詳細測試、打包與未完成項目見 [第二批紀錄](../verification/runs/RUN-20261106-102-SETTINGS-TRANSPORT.md)。
