# 1.0.2 設定、詞彙與 Ollama 傳輸驗證

更新時間：2026-11-06T10:59:25+08:00

## 已完成的自動化驗證

所有 Windows 測試均在 WorkstationX 執行。Mac 編輯前端並執行 TypeScript／Vite 編譯，未安裝或重啟任何輸入法。

| 項目 | 實際結果 | 日誌（WorkstationX 的 `%TEMP%`） |
| --- | --- | --- |
| policy-core | 105 單元＋5 契約＝110 passed、0 failed，exit 0 | `ckai-settings-core-tests.log` |
| AGY 傳輸＋Codex 跨模組整合 | 19 passed、0 failed，doc-tests 0，exit 0 | `ckai-102-transport-integrated.log` |
| 設定程式 | 9 passed、0 failed；預設忽略 1 個需網路的測試，exit 0 | `ckai-102-preferences-final-tests.log` |
| GitHub 實際唯讀查詢 | 另行執行該網路測試，1 passed、0 failed，exit 0 | `ckai-102-github-live.log` |
| 設定頁前端 | `npm ci --ignore-scripts`、`npm run build` 通過；Vite 提示單一 bundle 超過 500 kB，非編譯錯誤 | Mac 工具輸出 |
| 設定程式 Release | `cargo build --release --locked --bin ChewingPreferences --features tauri/custom-protocol`，exit 0 | `ckai-102-preferences-release.log` |

加上本節後續的原生測試後，合計 165 項無外網自動測試＋1 項 GitHub 唯讀連線測試。這些數字不包含尚未執行的 GUI 與實體按鍵。

設定程式於暫存來源快照建置，`src-tauri/Cargo.lock` 已帶回原始碼。Windows 沒有 npm 可執行檔；前端改在已有 Node 的 Mac 編譯再傳送資產，未安裝 Windows Node。Rustfmt 在此 Windows 工具鏈缺少 component，未將格式檢查記為通過。

## 審查與行為

- AGY 初版 `6746219` 經 Codex 要求去除原始底層錯誤文字、限制標頭，新增慢速滴流與標頭＋內文共用期限測試；修正 `cd63e83` 已整合，AGY 作者歷史保留。
- Codex 跨模組測試實際經過本機假 HTTP、JSON、前綴驗證及生命週期。HTTP 成功不會跳過編輯世代或明確接受檢查。
- 詞彙測試涵蓋 Unicode／大小寫保存、H₂O 與 H₂O₂、ASCII 輸入碼映射、原文優先、註解文本匯入、重複與損毀資料拒絕。
- 設定儲存使用同目錄暫存檔、原子替換、檔案鎖與舊版本比較；無效內容或另一視窗修改時不覆寫既有資料。
- 工程文本包含可複製的大模型提示詞，匯入只進入預覽，匯出僅包含已儲存自訂詞彙與固定說明。
- 更新查詢只讀本專案的正式 GitHub Release，語意版本比較避免降版；不自動安裝或執行下載檔。

## 未完成的驗收

- Zbook GUI、記事本開啟／複製提示詞、輸入法實體鍵盤、儲存後重開應用的讀取：NOT_RUN，交由使用者操作。
- Ollama 實際模型推論、TSF 候選接線與冷啟動效能：NOT_RUN，本批只完成核心與傳輸。
- 背景蒐集日常輸入或已送出前後文：未實作、未啟用。
- 正式 v1.0.2 發布與 Zbook 安裝：未執行。

## 共用來源與原生整合

- 外層 `8af1ec4`、preferences `113736a` 已 fast-forward 回 Windows 共用目錄；TSF 保持 `b5dc92e`，只重新連結更新的 policy-core。
- 以原生與設定 patch series 的 `-Check` 驗證完整 tree，兩者均通過。另從 preferences 固定上游基準實際重播兩個 patch，tree 精確匹配 `f8c78378ea1e7247aef61ad6de7a5faa829a0139`。
- Windows 共用原生目錄執行 `cargo test --locked -p chewing_tip_core -p chewing_tip -p tsfreg`：15＋9＋3＝27 passed、0 failed；doc-tests 0。日誌 `ckai-102-native-tests.log`。
- 同目錄 `cargo build --release --locked -p chewing_tip -p tsfreg`：exit 0。日誌 `ckai-102-native-build.log`。既有未使用項目與匯出符號連結器警告仍存在，非本批失敗。
- 共用 preferences 再以 `--release --locked --features tauri/custom-protocol` 建置：exit 0。前端 `index.html` 的 SHA-256 與 Mac 成品相同，避免使用舊設定資產。日誌 `ckai-102-shared-preferences-build.log`。
- AGY 接手乾淨打包 worktree、套件驗證、ZIP 與雜湊整理；不安裝、不註冊、不重啟輸入法。成品驗收另附 AGY 打包報告。
