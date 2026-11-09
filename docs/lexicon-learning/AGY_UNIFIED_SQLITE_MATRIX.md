# 統一詞彙管理 SQLite 架構規劃與合成驗證矩陣

> **最新範圍：延後方案。** 使用者改以離線手動匯入／匯出為近期方向；本矩陣保留歷史設計，不啟用週期、SHA 仲裁或強制上傳。手動匯入必須預覽與核准，不承接衝突時自動覆蓋授權。全部案例仍為 NOT_RUN。

> **文件代號**：CKAI-UNIFIED-SQLITE-PLAN-1109  
> **分支**：`research/unified-lexicon-sqlite`  
> **更新時間**：2026-11-09T00:33:00+08:00 (UTC+8)  
> **狀態**：**規劃階段（Planning-Only），未實作／未執行（NOT_RUN）**。  
> **重大排程與政策定案聲明（FINAL USER POLICY & FORCE UPLOAD）**：
> 1. **自動同步頻率上限：至多每週一次（AT MOST ONCE PER WEEK）**。下次嘗試時間為**上次嘗試（含失敗）起算 7 天（7 days after last attempt including failure）**。
> 2. **完全移除自動觸發**：徹底移除新增時防手震批次上傳（on-add / debounce）、每小時定時檢查（hourly）、啟動自動同步（startup，僅允許補跑一筆到期之每週補償）、以及資料夾變更監聽（folder-watch）。
> 3. **獨立顯式「強制上傳 (Force Upload)」按鈕**：
>    - **僅限使用者主動手動點擊（User-only）**，排程器（Scheduler）與 AI **絕對禁止觸發強制上傳（NEVER trigger force）**。
>    - 繞過每週頻率限制，繞過 `remote != base` 下載分支，以本機通過完整驗證之已核准快照直接覆蓋寫入固定檔名 `lexicon.sqlite`。
>    - 交付成功時，`base_sha` **一律更新為成功交付之捕獲快照雜湊（base ALWAYS updates to successfully delivered captured snapshot）**；僅有**清除 Dirty 標記受到世代檢查保護（ONLY clearing dirty is generation-conditional）**，交付期間產生的新編輯保留 Dirty。
>    - 寫入失敗完整保留本機狀態、`base_sha` 與 Dirty 狀態。
>    - 保留完整檔案完整性（Integrity）、確切固定檔名與安全原子寫入防護。
> 4. **一般手動同步維持雲端勝出（Ordinary Manual Sync Remains Cloud-Wins）**：介面上與「強制上傳」做明確視覺與流程區分。
> 5. **失敗絕不連續迴圈重試（Failure Must NOT Loop/Retry Continuously）**。
> 6. **一致性 SQLite 本機備份**：涵蓋 WAL 提交之資料，產出之獨立 SQLite 檔案**不包含亦不複製 SHM/WAL 臨時狀態檔案（Standalone file does NOT include/copy SHM/WAL state）**。
> 7. **無全域順序承諾**：同步軟體不保證 last writer 必定在雲端勝出，系統僅被動接受同步軟體所交付之任何合法固定檔案。  
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
  - 同步資料夾無伺服器端 CAS（Compare-And-Swap）支援；同步軟體不保證全域時間順序（No total ordering），亦不保證後寫入者必定全域勝出。使用者明確接受同步提供者最終交付之有效固定檔案內容，並接受覆蓋損失可能。
- **本機重新命名語意邊界**：
  - 本機以原子重新命名將新檔案交付給同步資料夾，**僅代表交付給本機端同步軟體（delivers to sync software），絕非雲端已完成上傳之證明（NOT proof remote uploaded）**。
- **一致性 SQLite 本機備份（Consistent Standalone SQLite Backup）**：
  - 本機快照之備份採用 SQLite 安全備份（如 `VACUUM INTO`），確保包含已透過 WAL 提交的所有完整資料；產出的獨立快照檔案**不包含亦不複製 SHM/WAL 等執行期狀態檔案（Does not include/copy SHM/WAL state）**。

### 1.2 三雜湊狀態機決策規則 (Three-Hash State Machine)
系統維護三個核心雜湊識別：
1. **`base_sha`**：最後一次成功接受（套用）或成功發布/交付之完整快照 SHA-256。
2. **`remote_sha`**：目前於同步資料夾中檢測並驗證合法之 `lexicon.sqlite` 完整快照 SHA-256。
3. **`local_sha`**：本機目前候選之最新快照 SHA-256。

#### 核心狀態轉換決策樹與世代防護：
1. **狀況一：`local_sha == remote_sha`**
   - **行為**：靜態無操作（`no-op`）。
   - **世代防護**：`base_sha = remote_sha` 一律更新；但**僅在比對檢查期間本機世代未變更（Generation Unchanged）的情況下才清除 Dirty 狀態**。若比對期間產生新編輯，本機必須保持 Dirty。
2. **狀況二：`remote_sha != base_sha`（雲端勝出，Cloud-Wins）**
   - **行為**：**全量套用雲端快照（Adopt cloud whole snapshot），直接捨棄尚未上傳之本機編輯（DISCARD unuploaded local edits）**。
   - **非同步套用編輯序列化（Async Adopt Serializes Edits）**：套用雲端快照為非同步作業，必須與本機編輯嚴格序列化；在套用確認完成之後才發生的本機編輯（after-adopt edits），**必須明確保持為 Dirty**。
   - **無干擾原則**：無行列合併、無本機重放、無強制衝突確認 UI、**不觸發回音上傳（NO echo upload）**。
3. **狀況三：`remote_sha == base_sha` 且本機處於髒狀態（`is_dirty == true`）**
   - **行為**：允許並觸發全量發布，將本機 `local_sha` 寫入同步資料夾取代 `lexicon.sqlite`。
   - **正確更新語意**：發布成功時，**`base_sha` 一律更新為所交付捕獲世代快照之雜湊（base ALWAYS updates to successfully delivered captured snapshot）**；**僅有清除 Dirty 標記受到世代檢查防護**：若交付期間產生了新編輯，本機保持 Dirty，下一週期重新比對 `remote` 與 `base`。
4. **獨立使用者「強制上傳」機制 (User Force Upload)**：
   - 僅能由使用者在獨立 UI 上手動點擊「強制上傳 (Force Upload)」觸發。排程器與 AI 絕對不得觸發。
   - 繞過每週排程頻率限制，繞過 `remote != base` 下載邏輯，將本機已核准之完整快照原子寫入覆蓋 `lexicon.sqlite`。
   - 成功交付後，`base_sha` 一律更新為該次交付快照之雜湊；交付期間之新編輯保持 Dirty。若寫入失敗，完整保留本機快照、既有 `base_sha` 與 Dirty 狀態。
5. **排程頻率與異常防護邊界（Cadence & Boundary Handling）**：
   - **排程頻率：至多每週一次（At Most Once Per Week）**：
     - 下次嘗試時間為**上次嘗試（含失敗）起算 7 天後（7 days after last attempt including failure）**。
     - 徹底移除 on-add debounce、hourly 與 folder-watch。
   - **休眠與錯過排程單次補跑（Missed Week Single Catch-up）**：
     - 錯過排程喚醒時**僅執行單次補跑檢查**，絕不重複累計。
   - **失敗絕不連續迴圈重試（Failure No Retry Loop）**：
     - 任何失敗立即中止，保留 Dirty，不進入重試迴圈，等待 7 天後下次排程或使用者手動觸發。
   - **一般手動同步 (Ordinary Manual Sync)**：
     - 提供使用者手動「一般同步」，不受每週週期限制，但維持標準雲端勝出（Cloud-Wins）邏輯。
   - **首次同步 (First Sync)**：
     - 有遠端檔案則直接採納；無遠端檔案則僅在使用者點選明確初始化時才進行發布。
   - **組字無逾時強制切換（No Timeout Forced Composition Switch）**：正在組字時，新快照一律在**閒置邊界 (idle boundary)** 生效，絕無逾時強制中斷。

### 1.3 核心輸入路徑與 libchewing (ec7f595) API 邊界
- **熱路徑不可變記憶體**：輸入法按鍵處理與候選字查詢僅存取已核准之記憶體快照，絕不進行磁碟 SQLite I/O。
- **中文專有名詞與符號映射分流**：
  - 中文專有名稱（包含人名、地名、機構名）配對完整驗證讀音，預設直接作為中文候選字投影。
  - 中文文字至符號映射舉例為「`水` -> `H₂O`」，與純 ASCII 觸發碼「`H2O` -> `H₂O`」各自獨立觸發，互不混淆。
  - 破音字（多音字）若缺讀音不自動補齊，須經明確驗證。
- **雙重路徑與 API 行為**：
  - 原生端 `isolated_editor`（傳入 `ChewingDev/chewing.dat` 衍生 `/v4/user_dict.csv`）與選單 `ID_HASHED`（直接開啟 `ChewingDev/v4/user_dict.csv`）雙重顯式來源需列入獨立審計。
  - 對齊 `ec7f595`：`UserDict::entries` 包含 banned 且省略 boost，匯出時顯式保留 banned 狀態；`unlearn_phrase` 單音節刪除、多音節 ban+清除歷史。

---

## 2. 合成驗證矩陣 (Synthetic Verification Matrix)

> **重要標記**：所有測試項目狀態一律為 **`NOT_RUN`**。

| 編號 | 測試場景與分類 | 前置條件與測試輸入 | 預期系統行為與驗證標準 | 驗證狀態 |
| :--- | :--- | :--- | :--- | :---: |
| **M01** | **下載不回音上傳**<br>*(download_no_echo)* | 偵測到同步資料夾中出現合規新版 `lexicon.sqlite`（`remote_sha != base_sha`），本機下載並套用 | 1. 本機全量套用新快照<br>2. `base_sha` 更新為 `remote_sha`<br>3. 本機標記為 Clean，**絕不反向觸發上傳寫入同步資料夾** | `NOT_RUN` |
| **M02** | **每週排程頻率上限**<br>*(weekly_limit)* | 本機在一週內多次新增詞條，且無手動觸發同步 | 1. **新增時不觸發任何自動上傳（無 on-add debounce）**<br>2. 自動排程嚴格限制**至多每週執行一次**<br>3. 未達每週排程週期前保持本地 Dirty 狀態 | `NOT_RUN` |
| **M03** | **發布中途編輯保持髒狀態與重檢**<br>*(publish_during_edit)* | 本機正在每週排程發布世代 $G_N$，期間使用者於本機新增詞條進入世代 $G_{N+1}$ | 1. 世代 $G_N$ 成功交付後，**`base_sha` 一律更新為 $G_N$ 雜湊**<br>2. 世代檢查發現已進入 $G_{N+1}$，**本機明確保持 Dirty 狀態**<br>3. 下一週期**必須重新檢查 remote/base 狀態**，不保證直接發布 $G_{N+1}$ | `NOT_RUN` |
| **M04** | **嚴格檔名限定防護**<br>*(fixed_name_only)* | 同步資料夾中同時存在 `lexicon.sqlite` 與 `lexicon (Conflict Copy).sqlite` | 1. 系統**嚴格只開啟並驗證 `lexicon.sqlite`**<br>2. **絕不讀取、絕不修改、絕不刪除衝突副本**，不報錯崩潰 | `NOT_RUN` |
| **M05** | **基礎版本衝突雲端勝出**<br>*(base_sha_conflict_cloud_wins)* | 本機有未上傳之編輯（`local_sha != base_sha`），但同步資料夾已被外部更新（`remote_sha != base_sha`） | 1. 判定雲端勝出（Cloud-Wins）<br>2. **全量套用遠端快照，直接捨棄本機未上傳編輯**<br>3. 不強制彈出衝突選擇對話框，不重放本機修改，清除本機 Dirty 標記 | `NOT_RUN` |
| **M06** | **讀取失敗阻斷發布**<br>*(read_failure_blocks_publish)* | 同步資料夾之 `lexicon.sqlite` 處於同步軟體寫入鎖定或位元組截斷狀態 | 1. 遠端讀取檢驗失敗<br>2. 系統**完整保留本機既有編輯與 Dirty 狀態**<br>3. **嚴格阻斷發布（blocks publish）**，防止覆蓋雲端檔案，待後續排程或手動處理 | `NOT_RUN` |
| **M07** | **首次同步採納現有遠端**<br>*(first_sync_adopts_remote)* | 本機全新設定同步資料夾，該資料夾中已存在合法之 `lexicon.sqlite` | 1. 首次同步自動全量採納該遠端快照<br>2. 更新本機工作庫並將 `base_sha` 設為 `remote_sha`<br>3. 保持 Clean 狀態，不執行覆蓋上傳 | `NOT_RUN` |
| **M08** | **缺遠端僅顯式初始化發布**<br>*(missing_remote_explicit_init)* | 本機設定同步資料夾，但資料夾內完全無 `lexicon.sqlite` | 1. 自動背景同步判定為無遠端，**不自動盲目發布**<br>2. 僅在使用者點選「初始化並發布至同步資料夾」時才建立初始檔案 | `NOT_RUN` |
| **M09** | **組字中途閒置邊界無逾時強制**<br>*(active_composition_idle_no_timeout)* | 正在鍵入注音音節處於活躍 preedit 組字中，同步資料夾下載新快照 | 1. 活躍組字緩衝區保持原狀，不中途變更候選詞<br>2. **絕不因計時逾時而強制切換組字**，一律在組字送出清空之閒置邊界生效 | `NOT_RUN` |
| **M10** | **手勢按鍵不吞鍵防護**<br>*(f_shift_no_swallowing)* | 處於中文組字狀態下，觸發 F 分層符號手勢或 Shift 英文交接，同時存在未完成音節 | 1. `entering_syllable()` 嚴格防護，不吞噬當前按鍵<br>2. 狀態機平順切換，不遺漏任何按鍵事件 | `NOT_RUN` |
| **M11** | **原生與共用排除分離**<br>*(native_shared_exclusion)* | 原生 libchewing 封鎖詞條「測試A」，共用庫獨立收納「測試A」 | 1. 共用詞庫排除清單與原生 `user_dict` banned 分開管理<br>2. 對齊 `ec7f595` 特性，不污染原生歷史與加權 | `NOT_RUN` |
| **M12** | **非法 Schema 架構拒絕**<br>*(invalid_schema_defense)* | 同步資料夾中 `lexicon.sqlite` 缺少關鍵資料表或欄位類型不符 | 1. Schema 檢驗失敗即刻拒絕<br>2. 記錄安全審計錯誤，絕不執行非預期 SQL<br>3. 核心維持前一版合格記憶體快照 | `NOT_RUN` |
| **M13** | **明確停用防舊資料復甦**<br>*(explicit_disable_no_revival)* | 使用者於設定中勾選停用共用詞庫，隨後重啟輸入法或應用程式 | 1. 記憶體投影保持為空，不查詢同步資料庫<br>2. 絕不因偵測到磁碟檔案存在而自動喚醒舊資料 | `NOT_RUN` |
| **M14** | **一致性獨立 SQLite 備份**<br>*(consistent_sqlite_backup_no_wal)* | 本機快照產生備份並驗證其結構無損性 | 1. 採用 SQLite 安全備份 API，包含已透過 WAL 提交的所有數據<br>2. **產出之獨立 SQLite 檔案不包含亦不複製 SHM/WAL 臨時狀態檔案** | `NOT_RUN` |
| **M15** | **錯過排程單次補跑**<br>*(missed_week_single_catchup)* | 電腦休眠 14 天（錯過兩次每週排程）後喚醒 | 1. 系統偵測到過期排程，**僅執行單次補跑檢查**<br>2. 絕不連續補跑兩次或觸發風暴<br>3. 下次排程時間更新為本次嘗試起算 7 天後 | `NOT_RUN` |
| **M16** | **失敗絕不迴圈重試與排程推進**<br>*(failure_no_retry_loop)* | 每週同步排程執行時遭遇網路斷線或同步檔案讀取錯誤 | 1. 記錄失敗日誌，保留本機 Dirty 狀態<br>2. **立即退出當前排程，絕對不進行連續迴圈重試**<br>3. 下次嘗試時間推進至本次失敗嘗試起算 7 天後 | `NOT_RUN` |
| **M17** | **一般手動同步維持雲端勝出**<br>*(normal_manual_cloud_wins)* | 使用者點擊「一般同步」按鈕，此時遠端存在新快照（`remote != base`）且本地有未發布修改 | 1. 立即觸發同步，不受每週排程限制<br>2. 維持雲端勝出（Cloud-Wins），採納遠端並丟棄本地修改，清除 Dirty | `NOT_RUN` |
| **M18** | **檢查寫入競態無全域順序**<br>*(check_write_race)* | 兩台裝置同時將快照寫入同步資料夾 | 1. 無全域寫入順序保證，不保證後寫入者勝出<br>2. 系統被動接受同步軟體最終交付之合法固定檔案，接受覆蓋損失 | `NOT_RUN` |
| **M19** | **中文名與符號映射及多音字分流**<br>*(chinese_name_vs_symbol)* | 同時存在：<br>1. 中文名「沈」（含破音字 `ㄕㄣˇ` / `ㄔㄣˊ`）<br>2. 中文文字符號映射「水」->「H₂O」 | 1. 未完整驗證之破音字不自動補齊或提權<br>2. 「水」->「H₂O」與純 ASCII「H2O」->「H₂O」各自獨立觸發，互不干擾 | `NOT_RUN` |
| **M20** | **Banned 狀態之匯出處理**<br>*(banned_export_state)* | 詞庫中存在被標記為 banned 的詞條 | 1. 對齊 `ec7f595` 特性，匯出時**顯式保留或標記其 banned 狀態**<br>2. 避免匯出重新載入後導致 banned 標記遺失 | `NOT_RUN` |
| **M21** | **使用者手動強制上傳**<br>*(manual_force_upload)* | 遠端快照已被其他裝置更新（`remote != base`），使用者點擊獨立「強制上傳」按鈕 | 1. 繞過每週限制，繞過遠端下載分支<br>2. 以本機已核准之完整快照原子覆蓋 `lexicon.sqlite`<br>3. 交付成功後更新 `base_sha` 為本機交付雜湊，清除 Dirty | `NOT_RUN` |
| **M22** | **強制上傳中途編輯防護**<br>*(force_during_edit)* | 正在執行手動強制上傳世代 $G_N$ 期間，使用者於本機新增詞條產生世代 $G_{N+1}$ | 1. 交付成功後，`base_sha` **一律更新為 $G_N$ 雜湊**<br>2. 世代檢查發現世代已遞增，**本機明確保持 Dirty 狀態** | `NOT_RUN` |
| **M23** | **強制上傳失敗保留狀態**<br>*(failed_force_preserves_state)* | 點擊「強制上傳」時遭遇磁碟滿或同步資料夾寫入鎖定失敗 | 1. 捕獲異常並提示使用者寫入失敗<br>2. **完整保留本機快照、既有 `base_sha` 與 Dirty 狀態**，不損毀檔案 | `NOT_RUN` |
| **M24** | **絕對無自動觸發強制上傳**<br>*(no_automatic_force)* | 排程器每週排程觸發或 AI 背景運作 | 1. 自動流程與 AI 邏輯**絕不具備呼叫強制上傳之權限**<br>2. 強制上傳僅可由使用者在專屬 UI 明確點擊觸發 | `NOT_RUN` |
| **M25** | **雜湊比對中途編輯保持 Dirty**<br>*(equal_hash_during_edit)* | 執行 `local_sha == remote_sha` 檢查期間，使用者恰好完成一筆本機編輯 | 1. `base_sha` 更新為 `remote_sha`<br>2. 偵測到世代已遞增，**不清除 Dirty 標記**，確保該筆編輯不被遺漏 | `NOT_RUN` |
| **M26** | **一般發布失敗保留 Dirty 狀態**<br>*(failed_publish_retains_dirty)* | 每週自動排程或手動一般同步發布時遭遇寫入失敗 | 1. 捕獲異常，發布中止<br>2. **本機明確保留 Dirty 狀態**，保留前一版 `base_sha` | `NOT_RUN` |

---

## 3. 安全與執行邊界聲明
- 本文件僅為架構分析與合成矩陣規格規劃。
- **無私有資料存取**：絕不呼叫 API 存取使用者私有資料或 `%APPDATA%\Chewing`。
- **無環境變更**：未開啟任何 GUI/IME、未變更產品程式碼、未修改主要分支（`main`）、未執行安裝、資料遷移或服務重啟。
- 本矩陣全部項目維持 `NOT_RUN`，待 Codex 完成主規劃文件後對齊推進。
