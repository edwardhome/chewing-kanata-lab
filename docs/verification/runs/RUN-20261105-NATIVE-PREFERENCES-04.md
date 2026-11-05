# 原生設定頁恢復驗證

更新時間：2026-11-05T18:55:22+08:00

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

使用者已回覆「設定頁與選字都正常」，確認本輪設定頁與選字驗收通過。這是使用者實際操作回報；未由代理執行 GUI、自動打字或音效測試，也不擴張為所有應用／按鍵情境皆通過。

## Agy 分工與驗收

已用既有 `agy --print` 對話方式派發原生頁面來源及隔離方案審查。首輪八分鐘逾時；縮小範圍後回覆成功。採納舊設定欄位相容、資料路徑與前端建置流程的檢查建議。

未採信其「原版沒有 GUI 詞庫編輯器」與「本專案必然改成不相容詞庫格式」的無證據結論。另行讀取公開 `windows-chewing-editor`，確認存在 React DictionaryEditor 與 CSV 詞條讀寫；本次未移植或啟動該編輯器。核心重載目前透過登錄設定比較，沒有把 Agy 假設的具名 IPC 說成現況。

## 交付狀態

十二份原生 patch 與一份設定頁 patch 已在 Windows 的新 worktree 從各自基準套用，完整 tree 均一致；兩組 -Check：PASS。過程抓到 Windows checkout 換行轉換造成設定頁 patch SHA 不符，已新增 -text 屬性並重新取出正確檔案後通過，未略過雜湊檢查。

Windows workspace check --locked、真實套件 preflight：PASS。已安裝 1.0.1；COM 與 current.json 指向 `versions/1.0.1-b5dc92e1fb8f`，DLL 版本 1.0.1.0，已安裝應用顯示 1.0.1。安裝前後 ChewingDev 偏好匯出 SHA-256 相同。安裝驗證 runner 最後曾因未載入檢查函式而報錯；另行載入 package guard 後，已完成安裝檔案雜湊與偏好保留檢查，沒有重複安裝。

- 封裝外層來源：`c5e30ca`。
- Windows ZIP SHA-256：`146ed216daeb78400b580dbf1df357056860054b6c4011428e172c1883c95a75`。
- 完整來源 ZIP SHA-256：`63e441f817f42ac7342571fc013c0f8a6c973fcfd49d6f4454d852ff82103101`。
- Windows 交付：`Documents\AI-Chewing\1.0.1`；備份：`Documents\AI-Chewing\verification-1.0.1`。
- Mac 保存：`/Users/defu/Documents/AI Chewing/1.0.1/`；只存 Windows 成品，未改動 Mac 輸入法。

使用者已確認新版設定頁與選字正常。另回報右下角仍開啟簡化滑桿視窗；實際檢查發現 Explorer PID 1720 載入的是 preview.20261105.4 的 DLL，而安裝指標已是 1.0.1。已說明儲存工作後登出再登入以載入新版；未強制終止 Explorer。新版 ID_CONFIG 直接開啟同版本的完整 ChewingPreferences.exe，F 延遲位於「其他設定」。右下角入口在重新登入後的實際結果尚未回報。1.0.0 發布資產保留。

## 共用 Windows 工作目錄

依使用者要求，改用 `C:\Users\Defu\Documents\gis-code\chewing-kanata-lab` 與 Agy 共用。原有 AI 修改先以完整 Git bundle、binary diff 與未追蹤檔備份，再保存在原生研究分支 `research/agy-ai-before-settings-20261105`（commit `e66ef6c`）；未混入已驗收版本。備份為 `Documents\AI-Chewing\shared-workspace-backup-20261105T185040`。共用外層、原生及設定頁各使用 `fix/shared-native-settings` 分支，檔案與本輪驗收來源一致。Agy 交接通知使用既有 CLI 對話；第一次回覆 503，未宣稱已收到確認。

共用路徑的原生 `cargo check --workspace --locked` 與設定頁 `cargo check --release --locked --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol` 均已實際執行並通過；不只是複製檔案。兩組驗收來源 tree 核對一致。
