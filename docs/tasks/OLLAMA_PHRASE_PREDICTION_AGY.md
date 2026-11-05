# Ollama 詞組預測：AGY 開發任務

更新時間：2026-11-05T19:26:45+08:00

使用者授權：整理 Ollama 詞組預測功能，切出獨立開發分支交由 AGY 開始思考、實作與編譯；AGY 的 Git 提交名稱改為 `AGY`。本任務不直接安裝、合併 main 或發布未驗收功能。

## 工作位置、分工與來源

- 唯一 Windows 工作目錄：`C:\Users\Defu\Documents\gis-code\chewing-kanata-lab`。
- 外層、`upstream/windows-chewing-tsf`、`upstream/windows-chewing-preferences` 均切到 `feature/ollama-phrase-prediction`。保留既有分支與安裝的 1.0.1。
- 起點：外層 v1.0.1/main `bd1c72eb003d45293b498c1b09914a08dd964dd8`，原生 `b5dc92e1fb8f974ed40c79b8bd31907fd2b2dd28`，設定頁 `ed957f4c6dc2fea261e608ead9589cd0719577a4`。
- AGY 負責方案、程式、單元／契約測試與 Windows 編譯。Codex 負責差異審查、隱私與原生行為驗收，使用者負責最終實鍵驗收。
- 原生 `research/agy-ai-before-settings-20261105` 的 `e66ef6c` 是舊 AI 實驗備份；可讀取差異與重用已確認安全的部分，禁止整份覆蓋或直接 cherry-pick 回退已驗收核心。
- 請讀 AGENTS.md、docs/development/NATIVE_PREFERENCES.md 與 PARITY_MATRIX.md。不得變更原始 Kanata、讀取既有正式新酷音私人詞庫、剪貼簿、文件、已送出文字或密碼欄。
- 這是實作工作，不是只有計畫。先做可編譯的第一階段，留下實際輸出，再逐步串接。

## 功能與使用者行為

| 功能 | 本輪要求 | 邊界 |
| --- | --- | --- |
| 繁中詞組續寫 | 完整、尚未送出的中文組字停頓後，提供一個短續接詞組 | 半個注音音節、空組字、游標不在末端時不預測 |
| 英文詞組續寫 | 英文／Shift 英文組字提供短續接；工程詞庫優先 | 不翻譯、不改大小寫、不覆蓋既有工程詞候選 |
| 原文保留 | 原文永遠是第一個且預設選中的候選 | 不能自動套用、送出或偷偷取代原文 |
| 明確接受 | 在獨立 AI 候選狀態用 Tab 切換、Enter 接受已選建議；Esc 忽略 | 未選 AI 時 Enter、空白鍵仍走原生行為；不得搶走原生選字選單 |
| 繼續輸入 | 普通字元、數字、退格、游標移動、F 動作取消 AI 候選並交回原生處理 | 按鍵只處理一次，不漏字、不重複送出 |
| 中止與過期 | 編輯、焦點／文件／中英模式變更、安全欄位、取消組字時失效 | 舊請求不得污染下一個組字或另一個應用 |
| 設定入口 | 在現有完整 ChewingPreferences 分頁新增 Ollama 狀態、模型、預測開關與測試連線 | 保留 F 滑桿、字型、空白鍵選字與原生所有分頁；預設關閉、不改使用者現值 |

MVP 續寫限制為工程上的初始規格，並非實測模型能力：

- 完整候選必須逐字保留原始前綴；不得先 trim、大小寫轉換或 Unicode 正規化再宣稱前綴一致。
- 英文追加最多四個單字且 32 個字元；繁中追加最多八個 Unicode scalar，優先短詞組，不接受換行或控制字元。前綴中既有標點、空白、數字與 Unicode 字元必須原樣保留。
- 回傳格式採 `{ "suggestions": ["完整前綴加續寫"] }`；最多一個 AI 預測候選，允許空陣列。只接受有效 JSON、非空追加與合法長度；丟棄重複、原文、前綴不符或不符合語言策略的結果。
- 合成案例：`今天晚上` 可建議 `今天晚上一起吃飯`；`machine` 可建議 `machine learning`。不得將 `今天晚上` 改成 `明天晚上`，也不得將 `USB` 改成 `usb`。
- 「我在去一次 → 我再去一次」屬校正，不是預測。校正保護規則保留，但不能混用續寫的接受路徑；先完成預測，再另列校正串接缺口。
- 模型輸出及組字中的文字一律視為不可信資料，不執行其指令，也不讓它更改系統提示、工具、路徑或端點。

## 請求生命週期與效能

1. 去抖起點 700 ms、連續請求最短間隔 2 秒；這兩個值與 F 長按預設 150 ms 無關。相同組字／模式／游標版本不重複請求。
2. 每個輸入法服務實例最多一個在途請求，等待項只保留最新的一個；不得在按鍵中建立無界佇列或每字元新開執行緒。若尚未跨應用共用推論槽，必須明列此限制，不能宣稱全系統只有一個請求。
3. 請求攜帶組字、選取範圍、焦點／文件、語言模式及取消世代的 token。回覆及接受當下都重新比對；不能只依賴一個可重複的文字字串。
4. TSF 按鍵、UI、COM 回呼不可同步等待網路或模型。工作執行緒只持有純資料；透過合法的宿主執行緒通知回來，不跨執行緒移動 TSF COM/UI 物件，不持有互斥鎖開視窗。
5. 初始連線上限 250 ms、整體暖機推論期限 2 秒；真正的總期限不能因多次讀取而無限延長。逾時、斷線、錯誤都回到原文，不宣稱預測成功；不得無條件延長期限掩蓋故障。
6. 冷啟動與模型載入測試由設定頁明確觸發，和打字路徑分開。可評估 keep_alive，但不要在未開啟功能時自動送入組字或預載模型。
7. 日誌只記錄代碼、耗時、計數與 token 世代，不記錄組字、提示詞、回覆或私人詞彙。品質驗證僅用本文件的合成案例。
8. 安全欄位／安全桌面／未知是否安全的上下文不啟動 AI，並取消已有請求。焦點失效後不顯示快取建議。

## Ollama 連線與模型

- Windows 唯一後端為本機 Ollama；不使用 fm、雲端 API 或遠端 fallback。固定 `http://127.0.0.1:11434`，不得跟隨導向非 loopback 的重新導向，不傳遠端模型或工具要求。
- 先以 `GET /api/tags` 查詢實際已安裝模型。不可把舊程式硬編碼的 `qwen3.5:0.8b` 當成已存在或最佳模型。模型名稱與預測開關使用本產品獨立設定區。
- 使用 `/api/chat`、`stream:false`、結構化 JSON 格式、低溫度與有限輸出。`think` 是頂層欄位，需依本機版本／模型能力確認，不沿用 `options.thinking` 的未驗證寫法。無該能力時省略，不能假裝已關閉思考。
- 使用正常 JSON 序列化與能正確處理 HTTP 狀態碼、Content-Length、分段回應及內容大小上限的客戶端；不要以手工拼接 prompt／HTTP 或 read_to_end + 短逾時的截斷資料當有效 JSON。
- 資料上限建議：送入組字最多 256 Unicode scalar，回應最多 16 KiB；超限直接拒絕，不能偷偷截短前綴後套用。
- 目前檢查 11434 未連上。可偵測既有 Ollama 可執行檔，並以 loopback 限制啟動開發用服務；記錄 PID 與啟動方式，不改防火牆、系統開機設定或遠端開放。沒有模型則記錄 BLOCKED_MODEL，先用假伺服器完成契約測試；不要自行下載大型模型。

官方資料（2026-11-05 核對）：[Chat API](https://docs.ollama.com/api/chat)、[列出模型](https://docs.ollama.com/api/tags)。API 文件不能代替實際本機版本測試。

## 現有程式需特別注意

- `tip/src/text_service/mod.rs` 目前 `// mod ai_client;`，不能僅改註解啟用就宣稱完成；舊 ai_client 的 80/200 ms 期限、手工 HTTP、模型寫死及候選重排不是完整詞組預測。
- `policy-core/src/text_assistance.rs` 已有文字保護與提示詞；目前 prediction 描述只允許英文，新增中文續寫時要分開驗證兩種模式，不得放寬校正的保護條件。
- `ChewingPreferences` 後端目前儲存時強制 `enable_ai=false`。完成生命週期與安全檢查之前保持此限制；之後才把明確 opt-in 正確接通，不能只畫一個不會生效的開關，也不能升級後自動打開。
- 原生候選選擇、中文閒置 Enter／Backspace、空白鍵選字、雙擊 Shift、F 長按及 Shift+F 短按須維持 1.0.1 行為。
- macOS AI-Bopomofo 只是行為參考，不得改動或安裝 Mac 輸入法。

## 分階段交付與必要測試

### 第一個檢查點：現在開始

1. 讀取任務與現有程式，寫簡短設計決策；先在新分支實際執行基線編譯。
2. 實作可測的請求／回應型別、前綴驗證、去抖／限流／取消與過期結果管理，以及可替換的 Ollama transport。優先完成這些可編譯程式，不停在研究報告。
3. 假 transport／本機假 HTTP 伺服器涵蓋：正常中文與英文追加、前綴大小寫差異、Unicode、空值、錯誤 JSON、超大回應、非 200、斷線、總期限、取消後晚到、A/B 焦點交錯、重複文字不同世代、同時多請求最新優先。不得啟動 GUI 或讀真實輸入內容。
4. 提交第一批 source code，作者及提交者名稱均為 AGY；回報 commit、檔案、測試數、建置退出碼與尚未串接的部分。

### 第二個檢查點：串接與設定

- 接入 TSF 安全的非同步發布與明確接受路徑，再接完整設定頁。
- 用已安裝的本機模型進行固定合成案例測試；分別報冷啟動／暖機延遲及成功／拒絕／逾時，無模型不得偽造實測。
- 提供實鍵驗收清單：空白選字、Enter／Backspace、F、Shift+F、雙擊 Shift、候選切換、Esc、焦點移動、切英文、關閉／重新開啟 AI；GUI 與實體按鍵未執行就標 NOT_RUN。

Windows 建置入口：

```powershell
$env:PATH += ';' + $env:USERPROFILE + '\.cargo\bin'
$env:CARGO_TARGET_DIR = Join-Path $env:TEMP 'ckai-agy-ollama-target'
cargo test --manifest-path policy-core/Cargo.toml --locked
Push-Location upstream/windows-chewing-tsf
cargo test -p chewing_tip -p chewing_tip_core -p tsfreg --locked
cargo check --workspace --locked
cargo build --release -p chewing_tip -p tsfreg --locked
Pop-Location
```

新增依賴時可有意更新 Cargo.lock，之後再用 --locked 驗證。Windows PowerShell 5.1 不支援 `&&`；每一步檢查 `$LASTEXITCODE`。原生編譯 stderr 可能只是進度，不要讓 PowerShell 把普通進度誤判失敗。未改設定頁就不重建前端；若修改設定頁，需 Node/Vite 前端加 Tauri 編譯，不能依賴過期 dist 宣稱新 UI 已建置。

## Git 與驗收交接

- 三個儲存庫的 local `user.name=AGY`；沿用既有 email，設定頁未有 email 時沿用原生儲存庫的既有值。不改 global、GitHub 帳號、登入、金鑰或歷史作者，不簽署提交。
- 使用 Conventional Commits，如 `feat(ollama): add bounded phrase prediction requests`。AGY 自己寫的程式須由 AGY commit；不可把本文件作者改成 AGY 來冒充它已交付程式。
- 原生與設定頁都是獨立 Git repo：分別 commit，然後把差異附加至外層新的 patch series 項目與更新的來源 tree。不得只 commit 外層文件而漏掉 source code。保留既有 patch 的原始位元組與 SHA-256。
- 禁止 push 到上游 Codeberg 或原生 repo 的任意 origin。若推送，只能將外層本任務 feature 分支推至已授權的 `edwardhome/chewing-kanata-lab`，不得推 main、tag 或發布。
- 正式結果存 `docs/verification/runs/RUN-20261105-OLLAMA-AGY-01.md`，簡要進度放同目錄的階段紀錄；不得包含使用者輸入內容、金鑰或憑證。
- 完成每階段後回覆 Codex：方案、實際修改、commit、測試／編譯命令與退出碼、服務／模型狀態、風險與未完成事項。Codex 審查前不安裝或取代使用者的 1.0.1。
