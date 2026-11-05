# Ollama 詞組預測：第一批分工

更新時間：2026-11-06T01:16:43+08:00

本文件縮小並取代舊任務中「AGY 一次實作所有模組」的本輪分工；功能與隱私規格仍以 [原始任務](OLLAMA_PHRASE_PREDICTION_AGY.md) 為準。使用者授權 Codex 與 AGY 分工實作、以 Git 協作。這一批只交付可測試的核心，不代表已能在輸入法中使用 Ollama。

## 檔案所有權

| 負責者 | 功能 | 專屬修改範圍 | 工作分支 |
| --- | --- | --- | --- |
| AGY | 中英文精確前綴／續寫長度驗證及單元測試 | `policy-core/src/prediction.rs`、`lib.rs` 的 prediction 匯出、自己的驗證紀錄 | Windows `feature/ollama-phrase-prediction` |
| Codex | 停頓、限流、單一在途請求、取消世代、回覆與接受時效 | `policy-core/src/prediction_session.rs`、本文件與自己的驗證紀錄 | Mac `feature/ollama-lifecycle` |
| Codex | 審查與整合 | AGY 提交完成後才調整共同 `lib.rs`，加入跨模組測試 | `feature/ollama-lifecycle` |

Windows 唯一來源目錄仍為 `C:\Users\Defu\Documents\gis-code\chewing-kanata-lab`。Mac 使用 `/Volumes/DATA/DATA/chewing-kanata-lab` 編輯及審查。不得碰兩個 nested repo、原生設定頁、Kanata、使用者詞庫或目前安裝版本。

## Git 交接

共同起點為外層 `5a44293`。AGY 不切換 Windows 工作分支，只提交自己的檔案，作者與提交者名稱均為 AGY，沿用既有 email，不簽署、不推送、不修改 global 設定。保留既有未追蹤舊驗證紀錄。

Codex 提交自己的模組後，以 Git bundle 讀回 AGY commit，審查再 cherry-pick，保留 AGY 作者。整合完成並確認 AGY 已停止修改後，再把驗收過的 Git 歷史同步回 Windows 共用目錄；不在代理編輯時切分支、不以檔案覆蓋代替 Git 整合。本輪不合併 main、不發布或安裝。

## 模組契約

AGY 提供 `PredictionLanguage::{TraditionalChinese, English}` 與 `validate_and_extract_continuation(prefix, candidate, language) -> Option<&str>`。成功只回傳追加部分；前綴位元組必須完全相同，不 trim、不正規化、不改大小寫。拒絕空前綴、超過 256 scalar 前綴、空／純空白追加、控制字元和 Unicode 換行分隔符。英文追加最多 32 ASCII 字元與四個單字；中文最多八個 scalar。

Codex 提供純邏輯 `PredictionSession`。時間由呼叫者傳入單調毫秒時鐘；工作者與 TSF／COM 不在此模組建立。每個服務實例只允許一個在途請求，取消不會提前釋放仍在運行的工作者名額。700 ms 停頓、2 秒最短請求間隔、2 秒結果期限，等待內容只保留最新快照。焦點、文件、模式、選取、編輯與取消世代皆參與判斷；相同文字經過修改也不能復用舊結果。

接線時先完成 HTTP／JSON／前綴驗證，再以原請求 token 呼叫 `complete`；候選需與 token 配對保存。使用者明確選擇後，再以最新上下文呼叫 `accept`，成功才附加驗證過的 suffix。不得因核心回傳成功而自動送字。

## 後續批次

1. Ollama transport：固定 loopback、模型查詢、結構化 JSON、16 KiB 回應上限、250 ms 連線與 2 秒整體期限；假伺服器涵蓋 HTTP 分段、錯誤、取消及超時。
2. TSF：安全欄位與宿主執行緒通知、原文優先、明確接受、原生按鍵與 F 行為不變。
3. 完整設定頁：獨立模型設定、預設關閉、測試連線與冷啟動分開。
4. 實際模型與實體按鍵驗收；未執行項目標 NOT_RUN，不以單元測試代替。

## 協作方式

AGY 沿用已驗證的常駐互動式 SSH 終端與同一對話，不每次另開 print 程序。Codex 以日誌與 Git 查驗進度，不操作瀏覽器 GUI；不把連線成功當成程式或測試完成。
