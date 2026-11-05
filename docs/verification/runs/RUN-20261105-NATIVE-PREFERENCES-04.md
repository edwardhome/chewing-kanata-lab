# 原生設定頁恢復驗證

## 原因與修復

1.0.0 的 `ChewingPreferences.exe` 是簡化 Win32 面板，只提供少量選項；核心仍支援空白鍵選字、字型與大小，但 UI 沒有入口。獨立登錄設定區也未帶入使用者既有值。WorkstationX 原設定的 ShowCandWithSpaceKey=1、DefFontSize=16、DefFontFamily=Segoe UI、CandPerRow=9，在 ChewingDev 區原先皆不存在。

已先匯出 ChewingDev 備份，再僅填入缺少的上述四個值；未改舊設定區或私人詞庫。備份位於 WorkstationX 的 `%LOCALAPPDATA%\Temp\ckai-settings-restore-20261105T180657`。

1.0.1 恢復上游 v26.6.0.1 原生設定頁，保留原作者與授權。改為本產品核心依賴、獨立設定及符號路徑、模組相對版本／公開字典路徑；保留 F 延遲，連接雙擊 Shift 開關／長壓門檻。設定匯入缺少新欄位時使用預設；不再忽略儲存錯誤。WebView2 為 UI 執行需求，WorkstationX 已安裝 156.0.4314.58。

## 來源

- 原生 commit：`b5dc92e1fb8f974ed40c79b8bd31907fd2b2dd28`；tree：`a80531d814906541b7a50b194b15a6b600f8592c`。
- 設定頁 commit：`ed957f4c6dc2fea261e608ead9589cd0719577a4`；tree：`841bb5fc12ca24cf77d1f379ae7500d54f1b75d0`。
- 原生基準加十二份 patch；設定頁基準 `141a10aa9e822ec4511b4bf47c21691c51ad6775` 加一份 patch。各自 series.json 記錄檔案雜湊。

## 已執行驗證

| 案例組 | 通過 | 失敗 |
| --- | ---: | ---: |
| Windows policy-core | 37 | 0 |
| Windows chewing_tip | 15 | 0 |
| Windows chewing_tip_core | 9 | 0 |
| Windows tsfreg | 3 | 0 |
| Tauri 設定後端，無 GUI | 2 | 0 |
| Windows 套件檢查 | 23 | 0 |
| 合計 | 89 | 0 |

前端 TypeScript／Vite production build、Windows Tauri release build（custom-protocol）、原生 DLL／tsfreg release build：PASS。前端仍有 bundle 大小提示；原生仍有 dead-code／unused-variable／LNK4104 警告，不宣稱無警告。Mac 未安裝 Cargo，policy 測試在 Windows 執行並通過，未宣稱 Mac policy 測試通過。

GUI、實體按鍵、候選字大小與滑桿互動：NOT_RUN，須使用者驗收。未執行 GUI、自動打字或音效測試。

## Agy 分工與驗收

已用既有 `agy --print` 對話方式派發原生頁面來源及隔離方案審查。首輪八分鐘逾時；縮小範圍後回覆成功。採納舊設定欄位相容、資料路徑與前端建置流程的檢查建議。

未採信其「原版沒有 GUI 詞庫編輯器」與「本專案必然改成不相容詞庫格式」的無證據結論。另行讀取公開 `windows-chewing-editor`，確認存在 React DictionaryEditor 與 CSV 詞條讀寫；本次未移植或啟動該編輯器。核心重載目前透過登錄設定比較，沒有把 Agy 假設的具名 IPC 說成現況。

## 交付狀態

完整來源重現、套件驗證、安裝與使用者驗收結果待本輪後續補記。1.0.0 發布資產保持不變，尚未發布 1.0.1。
