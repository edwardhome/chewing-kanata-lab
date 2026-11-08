# 統一詞彙管理 SQLite 架構規劃與合成驗證矩陣

> **文件代號**：CKAI-UNIFIED-SQLITE-PLAN-1109  
> **分支**：`research/unified-lexicon-sqlite`  
> **更新時間**：2026-11-09T00:30:00+08:00 (UTC+8)  
> **狀態**：**規劃階段（Planning-Only），未實作／未執行（NOT_RUN）**。  
> **重大政策定案聲明（FINAL USER POLICY）**：本政策為**最終確定政策，正式取代並廢棄先前所有髒狀態優先（dirty priority）及 CAS 提案**。全量同步採用「雲端勝出（Cloud-Wins）、丟棄未上傳本機修改（Discard unuploaded local edits）」之三雜湊模型，固定檔名為 `lexicon.sqlite`。  
> **基準**：AI新酷音 Windows 1.1.1、pinned libchewing `ec7f595`、AI-Bopomofo Mac release。  
> **責任分工**：Codex 負責主規劃文件及整體架構審查；AGY 負責本規格規劃與合成測試矩陣。未經實機執行的驗收與測試項目狀態一律標註為 `NOT_RUN`。

---

## 1. 最終確定同步政策與狀態機架構 (Final User Policy)

### 1.1 儲存路徑與固定檔名 (Fixed Filename & Folder Sync)
- **固定檔名**：使用者指定之同步資料夾內確切名稱為 **`lexicon.sqlite`**。
- **嚴格檔名限定 (Fixed Name Only)**：
  - 應用程式讀取時**僅認準確切的 `lexicon.sqlite`**。
  - **忽略衝突副本之嚴格定義**：對雲端同步軟體產生之衝突重新命名副本（如 `lexicon (Conflict Copy).sqlite`）**絕不讀取、絕不修改、絕不刪除（Never read, never modify, never delete them）**。
- **無雲端 CAS 與檢查/寫入競態接受**：
  - 同步資料夾無伺服器端 CAS（Compare-And-Swap）支援；使用者明確接受「檢查與寫入之間可能存在競態條件（Check/Write Race），以及可能發生的覆蓋遺失（Accepted Possible Loss）」。
- **本機重新命名語意邊界**：
  - 本機以原子重新命名將新檔案交付給同步資料夾，**僅代表交付給本機端同步軟體（delivers to sync software），絕非雲端已完成上傳之證明（NOT proof remote uploaded）**。
- **一致性 SQLite 本機備份包含 WAL（Consistent SQLite Backup Incl WAL）**：
  - 本機快照之備份機制採用 SQLite 原生安全備份（如 `VACUUM INTO` 或 Online Backup API），完整涵蓋 WAL（Write-Ahead Log）與 SHM 狀態，**絕非 JSON 導出或未鎖定之單獨主檔案複製（Not JSON or raw main file copy）**。

### 1.2 三雜湊狀態機決策規則 (Three-Hash State Machine)
系統維護三個核心雜湊識別：
1. **`base_sha`**：最後一次成功接受（套用）或成功發布之完整快照 SHA-256。
2. **`remote_sha`**：目前於同步資料夾中檢測並驗證合法之 `lexicon.sqlite` 完整快照 SHA-256。
3. **`local_sha`**：本機目前候選之最新快照 SHA-256。

#### 核心狀態轉換決策樹與世代防護：
1. **狀況一：`local_sha == remote_sha`**
   - **行為**：靜態無操作（`no-op`）。
   - **世代防護**：**僅在檢查期間本機世代未變更（Generation Unchanged）的情況下**，才清除 Dirty 狀態並更新 `base_sha = remote_sha`；若比對雜湊期間本機產生了新編輯，本機必須保持 Dirty。
2. **狀況二：`remote_sha != base_sha`（雲端勝出，Cloud-Wins）**
   - **行為**：**全量套用雲端快照（Adopt cloud whole snapshot），直接捨棄尚未上傳之本機編輯（DISCARD unuploaded local edits）**。
   - **非同步套用編輯序列化（Async Adopt Serializes Edits）**：套用雲端快照為非同步作業，必須與本機編輯嚴格序列化；在套用確認完成之後才發生的本機編輯（after-adopt edits），**必須明確保持為 Dirty**。
   - **無干擾原則**：
     - **不進行行列合併（NO merge）**。
     - **不進行本機動作重放（NO replay）**。
     - **不彈出強制衝突選擇對話框（NO conflict-choice requirement）**。
     - **不觸發回音上傳（NO echo upload）**：套用後清除本機髒標記，`base_sha = remote_sha`，絕不將剛下載的內容反向回寫同步資料夾。
3. **狀況三：`remote_sha == base_sha` 且本機處於髒狀態（`is_dirty == true`）**
   - **行為**：允許並觸發全量發布，將本機 `local_sha` 寫入同步資料夾取代 `lexicon.sqlite`。
   - **世代防護**：發布成功後，**僅在發布期間本機世代未改變時**才更新 `base_sha = local_sha` 並清除 Dirty；若發布期間本機產生了新編輯，發布完成後**本機依然保持 Dirty**，且在下一週期時**必須重新檢查 `remote` 與 `base` 的狀態**，絕非盲目保證發布新版本。
4. **異常防護與邊界處理**：
   - **讀取失敗防護 (Read Failure)**：若讀取同步資料夾之 `lexicon.sqlite` 失敗（檔案鎖定或不完整），系統**完整保留本機狀態並阻斷發布動作（blocks publish）**。
   - **發布失敗保留 Dirty**：發布過程遭遇寫入或鎖定失敗時，本機**完整保留 Dirty 狀態**以待後續排程重試。
   - **首次同步 (First Sync)**：
     - 同步資料夾若已存在合法的遠端 `lexicon.sqlite`，首次同步一律採納遠端（adopts it）。
     - 同步資料夾若無 `lexicon.sqlite`，**僅在使用者明確執行初始化操作時才進行發布（missing remote only explicit initialization publishes）**，絕不自動盲目空上傳。
   - **組字無逾時強制切換（No Timeout Forced Composition Switch）**：正在組字時，新快照一律在使用者送出或清空後的**閒置邊界 (idle boundary)** 套用，**絕不設定計時逾時強制中斷或切換組字狀態**。
   - **定時檢查**：乾淨啟動（Clean startup）、同步資料夾切換、防手震批次計時器與手動檢查時，若遠端檔案變更且驗證通過，執行全量套用；每小時排程在 Clean 狀態下**絕不上傳（hourly clean never uploads）**。

### 1.3 核心輸入路徑與 libchewing (ec7f595) API 邊界
- **熱路徑不可變記憶體**：輸入法按鍵處理與候選字查詢僅存取已核准之記憶體快照，絕不進行磁碟 SQLite I/O。
- **中文人名/專有名詞與符號映射分流**：
  - **中文專有/人名稱（Chinese Name/Proper Nouns）**：包含中文人名、地名、機構等專有名詞，配對完整驗證注音讀音後，預設直接作為中文候選字投影。
  - **中文文字至符號映射（Chinese Text -> Symbol Mapping）**：例如輸入完成之中文字「`水`」映射觸發工程符號「`H₂O`」；此機制與純 ASCII 輸入碼（如輸入英文字串「`H2O`」觸發「`H₂O`」）屬於各自獨立之觸發路徑，互不混淆。
  - **多音字與缺讀音防護**：破音字（多音字）若缺乏明確標記之完整注音，不盲目自動補齊；必須經過顯式讀音驗證方可入列。
- **雙重路徑與 API 行為**：
  - 記錄原生端 `isolated_editor`（傳入 `ChewingDev/chewing.dat` 衍生 `/v4/user_dict.csv`）與選單 `ID_HASHED`（直接開啟 `ChewingDev/v4/user_dict.csv`）之顯式來源差異，未來審計需涵蓋兩者。
  - 對齊 `ec7f595`：`UserDict::entries` 包含已被封鎖（banned）的條目且省略 boost 加權；共用詞庫匯出時必須顯式標示其 banned 匯出狀態；`unlearn_phrase` 單音節直接刪除、多音節 ban+清除歷史。

---

## 2. 合成驗證矩陣 (Synthetic Verification Matrix)

> **重大標記**：先前之 CAS 提案與髒狀態優先衝突選擇規劃已全數廢棄（SUPERSEDED）。本矩陣嚴格落實最終政策。所有測試項目狀態一律為 **`NOT_RUN`**。

| 編號 | 測試場景與分類 | 前置條件與測試輸入 | 預期系統行為與驗證標準 | 驗證狀態 |
| :--- | :--- | :--- | :--- | :---: |
| **M01** | **下載不回音上傳**<br>*(download_no_echo)* | 偵測到同步資料夾中出現合規新版 `lexicon.sqlite`（`remote_sha != base_sha`），本機下載並套用 | 1. 本機全量套用新快照<br>2. `base_sha` 更新為 `remote_sha`<br>3. 本機標記為 Clean，**絕不反向觸發上傳寫入同步資料夾** | `NOT_RUN` |
| **M02** | **閒置排程不上傳**<br>*(idle_no_publish)* | 本機處於 Clean 狀態（`local_sha == base_sha == remote_sha`），每小時排程計時器觸發 | 1. 檢查比對雜湊一致<br>2. **不執行任何上傳或檔案寫入動作** | `NOT_RUN` |
| **M03** | **發布中途編輯保持髒狀態與重檢**<br>*(publish_during_edit)* | 本機正在背景發布世代 $G_N$，期間使用者於本機新增詞條進入世代 $G_{N+1}$ | 1. 世代 $G_N$ 發布成功並更新 `base_sha`<br>2. 世代檢查發現本機已進入 $G_{N+1}$，**本機明確保持 Dirty 狀態**<br>3. 下一週期**必須重新檢查 remote/base 狀態**，不保證直接發布 $G_{N+1}$ | `NOT_RUN` |
| **M04** | **嚴格檔名限定防護**<br>*(fixed_name_only)* | 同步資料夾中同時存在 `lexicon.sqlite` 與 `lexicon (Conflict Copy).sqlite` | 1. 系統**嚴格只開啟並驗證 `lexicon.sqlite`**<br>2. **絕不讀取、絕不修改、絕不刪除衝突副本**，不報錯崩潰 | `NOT_RUN` |
| **M05** | **基礎版本衝突雲端勝出**<br>*(base_sha_conflict_cloud_wins)* | 本機有未上傳之編輯（`local_sha != base_sha`），但同步資料夾已被外部更新（`remote_sha != base_sha`） | 1. 判定雲端勝出（Cloud-Wins）<br>2. **全量套用遠端快照，直接捨棄本機未上傳編輯**<br>3. 不強制彈出衝突選擇對話框，不重放本機修改，清除本機 Dirty 標記 | `NOT_RUN` |
| **M06** | **讀取失敗阻斷發布**<br>*(read_failure_blocks_publish)* | 同步資料夾之 `lexicon.sqlite` 處於同步軟體寫入鎖定或位元組截斷狀態 | 1. 遠端讀取檢驗失敗<br>2. 系統**完整保留本機既有編輯與 Dirty 狀態**<br>3. **嚴格阻斷發布（blocks publish）**，防止覆蓋雲端檔案，待後續重試 | `NOT_RUN` |
| **M07** | **首次同步採納現有遠端**<br>*(first_sync_adopts_remote)* | 本機全新設定同步資料夾，該資料夾中已存在合法之 `lexicon.sqlite` | 1. 首次同步自動全量採納該遠端快照<br>2. 更新本機工作庫並將 `base_sha` 設為 `remote_sha`<br>3. 保持 Clean 狀態，不執行覆蓋上傳 | `NOT_RUN` |
| **M08** | **缺遠端僅顯式初始化發布**<br>*(missing_remote_explicit_init)* | 本機設定同步資料夾，但資料夾內完全無 `lexicon.sqlite` | 1. 自動背景同步判定為無遠端，**不自動盲目發布**<br>2. 僅在使用者點選「初始化並發布至同步資料夾」時才建立初始檔案 | `NOT_RUN` |
| **M09** | **組字中途閒置邊界無逾時強制**<br>*(active_composition_idle_no_timeout)* | 正在鍵入注音音節處於活躍 preedit 組字中，同步資料夾下載新快照 | 1. 活躍組字緩衝區保持原狀，不中途變更候選詞<br>2. **絕不因計時逾時而強制切換組字**，一律在組字送出清空之閒置邊界生效 | `NOT_RUN` |
| **M10** | **手勢按鍵不吞鍵防護**<br>*(f_shift_no_swallowing)* | 處於中文組字狀態下，觸發 F 分層符號手勢或 Shift 英文交接，同時存在未完成音節 | 1. `entering_syllable()` 嚴格防護，不吞噬當前按鍵<br>2. 狀態機平順切換，不遺漏任何按鍵事件 | `NOT_RUN` |
| **M11** | **原生與共用排除分離**<br>*(native_shared_exclusion)* | 原生 libchewing 封鎖詞條「測試A」，共用庫獨立收納「測試A」 | 1. 共用詞庫排除清單與原生 `user_dict` banned 分開管理<br>2. 對齊 `ec7f595` 特性，不污染原生歷史與加權 | `NOT_RUN` |
| **M12** | **非法 Schema 架構拒絕**<br>*(invalid_schema_defense)* | 同步資料夾中 `lexicon.sqlite` 缺少關鍵資料表或欄位類型不符 | 1. Schema 檢驗失敗即刻拒絕<br>2. 記錄安全審計錯誤，絕不執行非預期 SQL<br>3. 核心維持前一版合格記憶體快照 | `NOT_RUN` |
| **M13** | **明確停用防舊資料復甦**<br>*(explicit_disable_no_revival)* | 使用者於設定中勾選停用共用詞庫，隨後重啟輸入法或應用程式 | 1. 記憶體投影保持為空，不查詢同步資料庫<br>2. 絕不因偵測到磁碟檔案存在而自動喚醒舊資料 | `NOT_RUN` |
| **M14** | **一致性 SQLite 本機備份包含 WAL**<br>*(consistent_sqlite_backup_incl_wal)* | 本機快照產生備份並驗證其可還原性 | 1. 採用 SQLite 原生安全備份（如 `VACUUM INTO`），涵蓋 WAL/SHM<br>2. **絕非單純複製 raw 主檔案或 JSON**，確保備份庫結構無損一致 | `NOT_RUN` |
| **M15** | **發布失敗保留 Dirty 狀態**<br>*(failed_publish_retains_dirty)* | 本機嘗試將快照交付給同步資料夾時遭遇磁碟滿或檔案鎖定異常 | 1. 捕獲寫入異常，發布中止<br>2. **本機明確保留 Dirty 狀態**，待下一週期重新嘗試 | `NOT_RUN` |
| **M16** | **雜湊比對中途編輯保持 Dirty**<br>*(equal_hash_during_edit)* | 執行 `local_sha == remote_sha` 檢查期間，使用者恰好完成一筆本機編輯 | 1. 偵測到世代已遞增（Generation Changed）<br>2. **不清除 Dirty 標記**，確保該筆編輯不被遺漏 | `NOT_RUN` |
| **M17** | **防手震批次合併發布**<br>*(debounce_batch)* | 使用者連續快速新增 5 筆詞條（間隔 200ms） | 1. 防手震計時器重設，不為每筆單獨發布<br>2. 靜止後批次合併為單一快照發布一次 | `NOT_RUN` |
| **M18** | **檢查寫入競態接受損失**<br>*(check_write_race)* | 兩台裝置同時基於 base 檢查通過並寫入覆蓋 `lexicon.sqlite` | 1. 同步資料夾無 CAS，後寫入者覆蓋先寫入者<br>2. 先寫入者後續同步時判定遠端變更，**接受覆蓋損失並採納遠端** | `NOT_RUN` |
| **M19** | **中文名與符號映射及多音字分流**<br>*(chinese_name_vs_symbol)* | 同時存在：<br>1. 中文名「沈」（含破音字 `ㄕㄣˇ` / `ㄔㄣˊ`）<br>2. 中文文字符號映射「水」->「H₂O」 | 1. 未完整驗證之破音字不自動補齊或提權<br>2. 「水」->「H₂O」與純 ASCII「H2O」->「H₂O」各自獨立觸發，互不干擾 | `NOT_RUN` |
| **M20** | **Banned 狀態之匯出處理**<br>*(banned_export_state)* | 詞庫中存在被標記為 banned 的詞條 | 1. 對齊 `ec7f595` 特性，匯出時**顯式保留或標記其 banned 狀態**<br>2. 避免匯出重新載入後導致 banned 標記遺失 | `NOT_RUN` |

---

## 3. 安全與執行邊界聲明
- 本文件僅為架構分析與合成矩陣規格規劃。
- **無私有資料存取**：絕不呼叫 API 存取使用者私有資料或 `%APPDATA%\Chewing`。
- **無環境變更**：未開啟任何 GUI/IME、未變更產品程式碼、未修改主要分支（`main`）、未執行安裝、資料遷移或服務重啟。
- 本矩陣全部項目維持 `NOT_RUN`，待 Codex 完成主規劃文件後對齊推進。
