# 1.0.2-preview.2 中文輸入恢復

更新時間：2026-11-06T15:52:34+08:00

## 使用者回報與完成範圍

症狀為「切到中文仍只能輸入英文」。本輪加入原生選單與完整設定頁「其他設定」的 `Restart：恢復中文輸入`。這是安全重設輸入狀態，並非終止系統或應用程式、重新註冊輸入法或熱更新 DLL。

程式審查找到一條可解釋症狀的路徑：焦點切換遺漏 Shift key-up，舊邏輯把記住的 Down／Consumed 與目前 Shift 狀態做 OR。現已改為目前按鍵狀態，並在焦點切換重設殘留手勢。尚未於使用者電腦重現，因此不宣稱這就是其唯一根因。

- 原生來源：`acdbd7057d8dc48d57d596b3473c2ca56c3e4909`。
- 設定來源：`91e926ad2a89d2594db681dd7001f303e9cbf9b5`。
- AGY 初版恢復判斷保留作者為 AGY：`9594a78`；Codex 時間校正修正 `acdbd70`。
- 原生／設定 patch series 分別 17／4 個，從固定上游基準重播後完整 tree 精確匹配；SHA-256 均一致。

## 恢復契約與審查

- 明確點擊才寫入 `HKCU\Software\ChewingDev\InputRecoveryRequest` QWORD epoch 毫秒，30 秒有效；設定頁只顯示已送出，不冒稱已恢復。
- 個別原生實例辨識 token；零、重複、過期及未來值不執行。系統時間校正後的新有效 token 仍可執行。
- 只在已知可編輯脈絡及安全焦點處理。組字、注音、候選、ASCII／F 緩衝、待處理編輯或按住修飾鍵／F 時延後；不取消或提交使用者文字。
- CapsLock 設定若鎖定英文則延後，不改偏好。設定頁提示使用者先切回對應中文狀態。
- editor 重建先建立新實例，保留全半形並同步語言。設定重載與 thread focus 均延後到沒有待處理輸入。
- TSF compartment 寫入在可重入區進行；通知回呼只讀取狀態，不再反寫。失敗的恢復不保留無期限重試旗標，需使用者重新要求。
- 未讀取正式新酷音個人詞庫、未變更 Kanata、未啟用 Ollama 或背景輸入紀錄。

## 實際驗證

| 項目 | 結果 | 證據 |
| --- | --- | --- |
| Mac 前端 `npm run build` | PASS | TypeScript、Vite 通過；既有大 bundle 警告 |
| Windows 設定測試 | 9 passed，0 failed，1 ignored | `%TEMP%\ckai-recovery-preferences-tests.log` |
| Windows 設定 Release | PASS，exit 0 | `%TEMP%\ckai-recovery-preferences-final-build.log` |
| Windows 原生測試 | 28＋9＋3＝40 passed，0 failed，doc-tests 0 | `%TEMP%\ckai-recovery-native-tests.log` |
| Windows 原生 Release | PASS，exit 0 | `%TEMP%\ckai-recovery-native-build.log` |
| 原生與設定 patch 重播 | PASS，完整 tree 一致 | native `5efff15276273d4c57faab9a133e9fc1e22d7091`；preferences `70e24416db653ab76939ff1cbad591da0a5e3581` |

本輪合計 **49 項通過**。GitHub 實際查詢測試預設忽略，沒有重新宣稱上一批的連線測試為本輪結果。原生新增 3 項 Shift 回歸、10 項恢復 token 測試；核心與傳輸未變更，上一批完整測試仍見原報告。

設定前端在 Mac 建置後移交 Windows，最終 index.html SHA-256 為 `536acc7b1ac57acb46aabf9be866b98dffd871013e09e840833d8f17c7fafd6a`，兩端一致。原生在固定 commit 的驗證 worktree 建置，避免 AGY 尚在編輯的檔案影響驗收；不是另一套開發來源。

首次 PowerShell 執行因 ErrorActionPreference Stop 把 Cargo 的 stderr 編譯訊息當成終止錯誤，未算成功；改用原生命令結束碼後重新執行。保留既有 unused 與 LNK4104 警告，未偽稱零警告。

## 尚未驗收

- Zbook 安裝、右下角選單、設定按鈕實際恢復及實體按鍵：NOT_RUN，交由使用者操作。
- 中文空組字 Enter／退格、Shift 切換焦點、F 長短按、雙 Shift、CapsLock、密碼與不可編輯脈絡：需手動回歸，詳見 [驗收案例](../../tasks/INPUT_RECOVERY_102.md)。
- DLL 在應用程序內無法由此按鈕熱更新；安裝新包後須重新開啟測試用應用。
- 未安裝、未終止應用或系統程序、未發布正式 v1.0.2、未合併 main。
