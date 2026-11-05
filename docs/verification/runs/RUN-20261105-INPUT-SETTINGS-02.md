# 中文閒置編輯鍵與設定視窗重入修正

更新時間：2026-11-05T14:23:04+08:00

## 使用者回報與根因

使用者確認：中文模式沒有組字時，Enter 無法換行、Backspace 無法刪除已送出文字；組字中不受此回報影響。設定顯示預設 150ms，但拖曳滑桿仍會卡死。

1. 基準 `053d31ba` 的 OnTestKeyDown 在沒有組字時放行非 Unicode 控制鍵。後續版本把這一區塊限制於英文模式，使中文閒置狀態仍宣告接收 Enter／Backspace。修正於自訂 F／ASCII 與明確 keybinding 處理後，檢查 engine 空白且沒有 TSF 組字，讓非文字控制鍵回到應用。組字中的確認、刪除與 F 內退格保留。
2. 舊設定程式在持有 APP_STATE 的 Mutex 時呼叫 SetWindowTextW／SendMessageW／MessageBoxW；同步重畫進入 WM_CTLCOLORSTATIC／BTN 又取相同 Mutex，造成可重現於來源控制流的重入死鎖風險。修正為先複製狀態、釋放鎖，再呼叫 UI；僅更新延遲時短暫持鎖。滑桿、儲存、重設與資源清理一起修正。這是來源推導，未將使用者所稱當機斷言為已取得 crash dump 的程序崩潰。

## 來源與套件

- 外層：`b4b161e982f01c42c0555ebb16f0d6ed6809b370`，`fix/windows-input-settings-regressions`。
- 原生：`4a38766f59a96cb4c3eb5d34dde1c122db0a6d5d`，`fix/idle-edit-settings-reentrancy`。
- 十份 patch 從固定基準重放，完整 tree：`2d0f4bf0e865969ac4704ab8e87448ca4a4f762b`。
- Windows x64 版本：`1.0.0-preview.20261105.4`。
- ZIP SHA-256：`999a845bb05a525f22adf7d35e776144967b8e2e904edb9399d816c04e186017`。

## 本輪實際驗證

| 項目 | 結果 |
| --- | --- |
| chewing_tip | 15 PASS，新增 2 個控制鍵放行／組字保留案例 |
| chewing_tip_core | 8 PASS |
| tsfreg | 3 PASS |
| ChewingPreferences | 1 PASS，模擬巢狀 UI 回呼可重新取鎖、更新及重設狀態；不開視窗 |
| package guard | 19 PASS，Windows PowerShell 5.1 |
| 本輪案例合計 | 46 PASS，0 失敗、0 跳過；0 doctest |
| 四個原生交付元件 release build | PASS；既有 dead-code 與 LNK4104 警告仍存在 |
| 十份 patch 重現／SHA 與髒工作目錄拒絕 | PASS |
| 真實套件 ValidateOnly | PASS |
| .3→.4 版本目錄升級／註冊 | PASS；current.json 與 COM DLL 指向新版本，舊 DLL 檔案保留 |
| 既有上游註冊、預設輸入法與語言清單 | 安裝前後比對相同 |
| 解除安裝 ValidateOnly | PASS；沒有真正解除安裝 |
| 中文無組字 Enter／Backspace、設定滑桿 | PASS（使用者實機回報「兩項都恢復正常」） |
| 儲存／重設、其他應用與其餘鍵盤清單 | NOT_RUN，未取得逐項回報 |

先前 policy-core 37 與 C 契約 14 個案例的來源未修改，本輪未重跑，不計入上表。未啟動 GUI、自動打字、音訊或修改正式新酷音私人詞庫／Kanata；Mac 輸入法未安裝或重啟。

## 設定介面回退評估

上游基準沒有本次新增的 crates/chewing_preferences，設定入口是 `chewing-preferences://config`。這個協定啟動的另一套設定程式需另行稽核，不能僅把入口改回去就宣稱保留 ChewingDev 隔離。

Codex 建議先採本次最小修正；輸入核心保留上游行為，僅修正閒置按鍵的攔截邊界，無需整體撤掉 F／英文組字功能。若新設定視窗仍不穩，可先移除滑桿改用簡單固定選項，再評估將上游設定介面移植到獨立身分／偏好路徑；不可直接呼叫未稽核的既有協定 handler 或改寫正式新酷音設定。

Antigravity 已回覆（conversation `75f01f99-6d39-41ec-970d-5ce1d89e8f58`）：同意優先採最小修正，不建議整體回退；如未來採上游設定入口，應使用預覽版私有協定、ChewingDev 設定鍵、路徑及獨立 GUID。第一輪讀取來源逾時，第二輪由 Codex 補充已核實的程式證據後得到結論，屬來源審查建議，不是獨立 GUI 驗收。

Codex 驗收：其鎖重入與中文控制鍵分析符合所查原始碼；回覆額外提及 WebView2／IPC 架構未獲此次程式證據證實，不採為已確認事實。使用者隨後明確回報兩項問題皆恢復正常，因此目前維持修正版，沒有執行設定介面的整體回退。

## 人工驗收

先儲存工作再重新開啟測試應用；已載入的舊 DLL 不會因註冊更新自動替換。關閉舊設定視窗後，直接執行 `Documents\AI-Chewing\1.0.0-preview.20261105.4\ChewingPreferences.exe`，避免舊應用的選單再次啟動 .3 設定程式。

- 中文無組字：Enter 換行，Backspace／Delete 編輯文件，方向鍵／Tab 交回應用。
- 中文有組字：Enter 確認、Backspace 刪除組字；F 內退格不刪到手勢之前文字。
- 滑桿拖曳、重設 150ms、儲存與關閉對話框保持回應；重新開啟後確認已儲存值。

原始測試／建置日誌存於本機交付目錄 verification；登錄前後備份僅留 Windows 私人驗證目錄，不提交到公開倉庫。
