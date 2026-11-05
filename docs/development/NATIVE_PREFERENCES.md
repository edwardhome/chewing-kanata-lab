# 原生設定頁恢復

AI新酷音 1.0.1 使用 `windows-chewing-preferences` 的原生 React／Fluent UI／Tauri 設定頁，取代先前功能不完整的 Win32 簡化面板。上游基準為 v26.6.0.1，commit `141a10aa9e822ec4511b4bf47c21691c51ad6775`；此版本的欄位與已鎖定的輸入法核心相容。保留上游授權、歷史與關於頁作者資訊。

恢復打字行為、空白鍵選字、字型／大小／色彩、候選列與頁面、鍵盤配置、自訂快捷鍵、特殊符號、快捷符號及設定匯入／匯出。其他設定頁保留 F 長按延遲，預設 150 ms；雙擊 Shift 的開關與長壓門檻接回現有政策，仍需實體按鍵驗收。AI 與上游自動更新維持關閉。

## 建置與重現

1. 依現有原生 patch series 準備 `upstream/windows-chewing-tsf`，執行 `scripts/apply_patches.ps1 -Check`。
2. 將公開來源 `https://codeberg.org/chewing/windows-chewing-preferences.git` clone 到 `upstream/windows-chewing-preferences`；從上述基準建立獨立 `feature/*` 分支。不要重設已有修改的工作目錄。
3. 執行 `scripts/apply_preferences_patches.ps1`，使用清單內的 SHA-256 驗證每份 patch，套用後比對完整 tree。
4. 前端使用 Node.js 22+ 與 `npm ci --ignore-scripts --no-audit --no-fund`、`npm run build`。可在 Mac 編譯並將 `dist/` 複製到 Windows 的相同來源目錄；不需在 Windows 安裝 Node。
5. Windows 使用 `cargo build --release --locked --manifest-path src-tauri/Cargo.toml --bin ChewingPreferences --features tauri/custom-protocol`。`custom-protocol` 必須啟用，才能使用內嵌正式前端而非開發伺服器。
6. `cargo test --release --locked --manifest-path src-tauri/Cargo.toml --lib --features tauri/custom-protocol` 僅測試後端，不啟動 GUI。
7. 封裝時，額外傳入 `build_package.ps1 -PreferencesDirectory <來源目錄> -PreferencesExecutable <已建置的 ChewingPreferences.exe>`；三個來源工作目錄均須已提交且乾淨。manifest 記錄設定程式 commit 與 WebView2 需求，附上原版設定頁的授權檔。

原生 workspace 已停止建置舊簡化面板；其歷史來源保留。不要從舊建置目錄挑選同名 EXE。安裝前檢查 WebView2 執行階段，缺少時在變更輸入法註冊前停止。套件不自動下載或安裝瀏覽器元件。

## 資料邊界與已知範圍

設定僅寫 `HKCU\Software\ChewingDev`；符號檔僅位於 `%APPDATA%\ChewingDev`，內建字典與版本從 EXE 同目錄讀取。設定程式使用自己的 Tauri identifier，不註冊上游 URI。舊設定匯入缺少新欄位時使用安全預設，儲存失敗顯示錯誤且不關閉視窗。

1.0.0 的設定命名空間隔離沒有帶入使用者既有的空白鍵／候選字偏好。本次在 WorkstationX 備份後，僅將原有四項已確認偏好填入缺少的值；不自動覆寫新設定，不搬移或讀取私人詞庫。

這份恢復涵蓋設定頁及其符號文字編輯頁，**不包含獨立的詞庫 GUI 編輯器**。公開 `windows-chewing-editor` 確有 React GUI 與 CSV 詞條編輯實作；不能將它說成只有記事本。移植前仍需核對字典探索路徑、相容格式、寫入保護與獨立命名空間，不能直接啟動未隔離的上游 EXE。
