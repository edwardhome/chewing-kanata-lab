# 1.1.0：組字銜接與共用工程候選

狀態：已授權交付 AGY；不得混入正式 1.0.2。

更新時間：2026-11-06T17:58:40+08:00

## 使用者要求

1. 已驗收內容發布為正常版本，由 Codex 完成 1.0.2 發布。
2. 注音組字時可以使用 F 數字／符號分層；確定分層輸出後，直接確認當前顯示的中文字，再接續數字符號，不額外要求選字。
3. 注音組字中進入 Shift 連續英文輸入時，先送出當前中文，不吞掉組字；保留既有 Shift 放開後連續小寫的 Word session 行為。
4. 優化工程詞彙選單，讓中文與工程候選共用新酷音原生選字機制。

## 分工與來源

- Codex：正式發布、架構決策、Git／patch 整合與獨立驗收。
- AGY：依序實作 A、B、C；每階段完成測試與提交後回報，不跨階段擅自擴張範圍。
- Windows 共用根目錄：`C:\Users\Defu\Documents\gis-code\chewing-kanata-lab`。
- 開發分支：外層、原生 TSF、preferences 都使用 `feature/1.1.0`，不得切回或修改正式發布 worktree。
- 1.0.2 原生基線 `f342e38`；preferences 基線 `a9b2243`。正式版僅調版本，與已驗收 preview.2 行為相同。
- AGY 提交使用 `git -c user.name=AGY -c commit.gpgsign=false commit`，保留現有 repo-local email；Conventional Commits。只 add 自己的確切檔案，不使用 git add .。
- 不 push、不合併 main、不安裝、不註冊、不終止使用者應用、不改 Kanata、不讀正式 Chewing 私有詞庫。
- AGY 使用自己預設 `target` 或 `%TEMP%\ckai-agy-110-target`；禁止使用 Codex 正式發布的 `%TEMP%\ckai-native-codex-target`。
- 既有未追蹤報告保留。若檔案已有別人未提交修改，先回報，不覆寫。

## A：可靠組字銜接、F 分層與 Shift（立即實作）

### 所有權

僅修改 nested `upstream/windows-chewing-tsf`：`tip/src/text_service/chewing.rs`、`edit_session.rs`、`kanata_layer.rs`，必要的同目錄純邏輯 helper／模組宣告與測試。可唯讀 policy-core；本階段不得修改外層 policy-core、preferences、版本號、安裝及打包腳本。

### 先修正交接基礎

`set_composition_string` 目前會用 `cell.replace` 合併尚未執行的 pending edit，可能把先前 commit 一併蓋掉。Shift 路徑又分別呼叫非同步 `insert_text`、清空 preedit、顯示英文。不能只新增一個 commit 呼叫就視為修好。

建立可測試、有順序的組字交接：待提交文字不可被後續 preedit 更新覆蓋，連續兩次提交保留順序且只寫一次。先完成 TSF 寫入再確認可丟棄舊 engine 狀態；審查 SetText 失敗仍繼續移動 range／ack 的路徑。若失敗，保留可恢復狀態，不啟動一個已吞掉中文的新英文 session。

中文內容以當前畫面已解析的組字為準，不重新排序或偷選另一候選。若只是未完成注音，engine 無法產生可提交結果時，保留原組字，不猜字、不清空。

### F 行為

- test-keydown 與實際 keydown 兩處 F 條件都要調整，不能只移除單一 is_empty 判斷。
- F keydown 只是開始判斷手勢，不提前提交中文。
- 確認數字／符號分層手勢後，將目前中文與分層輸出按順序交接；F keyup 不得再重複提交中文。
- 短按 F 仍是原注音／字母，短 Shift+F 仍遵守原有大寫與英文行為；保留快速注音 F chord、直接英文及既有 ASCII 組字中的 F 分層。
- 取消或沒有產生分層輸出的手勢不確認中文字；F 退格刪光與按鍵重複需明確處理。

### Shift 行為

- 只有第一個實際英文字母觸發交接，單純按下／放開 Shift 不送出中文。
- 中文須完整保留、送出一次，再開始既有 ShiftLetterSession。
- 保留既有首字母、持續大寫／Word mode 和工程大小寫邏輯；不將所有英文字母強制小寫。
- Shift 放開後可持續輸入小寫並保留英文編輯、選詞、F 分層行為。
- 不破壞雙 Shift 切換、空組字 Enter／退格、Restart 與安全脈絡旁路。

### 必要驗證

- 中文＋F 長按數字：中文一次、數字符號順序正確；短 F 不提前確認。
- 中文＋Shift 首字母＋放 Shift＋後續小寫：中文不消失，英文可編輯。
- 下一鍵先於 DoEditSession、連續兩個提交、模擬 COM SetText 失敗：不遺失或重複文字。
- 候選已開啟、未完成注音、F 退格刪光、按鍵重複、焦點及模式／安全狀態變更。
- 先執行有意義的純邏輯測試，再跑 Windows native Cargo tests；不可把人工測試標成 PASS。

完成 A：提交 nested native，回報 commit、實際測試數、尚未驗證項目及任何設計限制。報告只能寫 `docs/verification/runs/RUN-20261106-AGY-110-A.md`（先不提交外層）。然後停在同一個互動工作階段，等待 Codex 放行 B，避免發布中共享依賴變更。

## B：工程詞彙唯一資料來源與候選模型（A 驗收後）

### 共用設計決策

目前 ASCII 與中文已使用同一個 CandidateList 視窗，但模型、分頁和選取動作分離。重用這個原生視窗，統一候選 snapshot，不另開工程浮動視窗。

候選資料應區分：穩定 id、真正要提交的文字、註解、來源、選取動作、組字世代／目標範圍。動作至少包含 SelectNative(original_index) 與 SelectEngineering(entry_id, replacement_scope)。不可將混合列表下標直接交給 chewing_editor.select。

- 原生候選保留順序、頁數設定、空白鍵開選單、選字鍵、字型／大小。
- 工程結果標示來源與註解，但註解不能進入提交文字或原生 unlearn。
- 保留原文與原生候選，不自動接受工程候選；去重後維持穩定順序。
- 工程項目不可觸發原生詞庫刪除／unlearn；原生選取須透過原始索引。
- 編輯、游標、焦點、語言模式改變後讓舊 snapshot 失效。

### 中文查找

同一個使用者詞彙 catalog 供英文輸入碼與中文別名查找。新增明確的 `zh_aliases`，例如 `input: CH4, output: CH₄, annotation: 甲烷, zh_aliases: [甲烷]`。註解只顯示，不能自動當查找詞，也不讓 Ollama 默默新增映射。

第一版中文查找僅對「完整目前未提交的中文組字」做精確別名匹配，避免缺少可靠選詞範圍時把整句改掉。例如組字「甲烷」可列出 CH₄；「甲烷濃度」不能整句換成 CH₄。若後續要支援句中詞，先建立經驗證的 engine 範圍映射與 UTF-16 游標處理。

沿用現有字數／檔案大小、控制字元限制、鎖定、revision 比較和原子儲存。JSON v1 與三欄 TSV 必須能匯入；若使用 v2／第四欄別名，必須有格式版本、舊檔備份、遷移預覽及回退說明，不偷偷破壞舊檔。不得放寬 ASCII input 欄位來混塞中文。

所有權候選：`policy-core/src/user_engineering.rs`、`technical_lexicon.rs`、新增純候選模型與測試。先提交這個可測試契約，通過後再交付 C 接線。

## C：原生候選與設定頁接線（B 驗收後）

- 同一原生 CandidateList 接上 source-aware 選取、分頁、去重、註解呈現。
- 設定表單加入「中文別名」，工程詞彙搜尋／篩選、重複提示與候選預覽；保留現有新增、刪除、匯入預覽、匯出。
- 批次文本與給大模型的提示詞同步更新，清楚區分輸入碼、輸出、註解與中文別名，仍需使用者檢查後儲存。
- 測試 v1 遷移、H2O／H₂O／H₂O₂、中文完整精確匹配、混合候選跨頁與原生索引、註解不提交、工程不 unlearn、失效 snapshot 及非 BMP 游標。

## 1.1.0 驗收界線

本文件是開發契約，不是完成或發布宣告。Codex 審查 Git diff、重播 patch、Windows 建置與測試後提供測試包；Zbook 的安裝、移除與真實按鍵由使用者操作。未有實測證據的 GUI 行為一律 NOT_RUN。Ollama 雲端／背景記錄或已提交文字擷取不在本輪範圍。
