# 統一詞彙管理 SQLite 架構規劃與合成驗證矩陣

> **文件代號**：CKAI-UNIFIED-SQLITE-PLAN-1109  
> **分支**：`research/unified-lexicon-sqlite`  
> **狀態**：**規劃階段（Planning-Only），未實作／未執行（NOT_RUN）**。  
> **重大修訂聲明**：使用者最新指示已**正式取代並廢棄先前之 CAS / 雲端伺服器 API 方案（Prior CAS / Cloud API proposal explicitly SUPERSEDED）**。改採純使用者指定之本機同步資料夾與固定檔名完整快照取代模型。  
> **基準**：AI新酷音 Windows 1.1.1、pinned libchewing `ec7f595`、AI-Bopomofo Mac release。  
> **責任分工**：Codex 負責主規劃文件及整體架構審查；AGY 負責本規格規劃與合成測試矩陣。未經實機執行的驗收與測試項目狀態一律標註為 `NOT_RUN`。

---

## 1. 統一詞彙管理架構與職責劃分

### 1.1 同步資料夾與固定檔名完整快照模型（Folder Sync & Fixed Filename Snapshot）
1. **同步邊界（Folder-Only, No Cloud API / CAS Prerequisite）**：
   - 僅透過使用者於本機指定之 Google Drive、iCloud Drive 或類似之雲端硬碟本機同步資料夾進行檔案交換。
   - 固定檔名：**`unified-lexicon.sqlite`**（暫定名稱）。
   - **讀取邊界**：應用程式僅讀取確切的固定檔名 `unified-lexicon.sqlite`，**一律忽略雲端同步服務自動產生之衝突重新命名副本（如 `... (Conflict Copy)`）**，不承諾保存或合併衝突副本。
   - **整體覆蓋（Whole-File Replacement，No Per-Row Merge）**：不進行行列級別之差異合併（No per-row merge）。使用者明確接受全量檔案覆蓋可能帶來的覆蓋遺失風險。
   - **無強制衝突選擇對話框（No Mandatory Conflict-Choice UI）**：接收到合規的新快照時，依既定政策可直接全量套用覆蓋本機未發布之修改；可選支援有界之本機可復原快照（Bounded Recoverable Local Snapshot）以供後續補救，但絕不強制彈出阻斷式衝突確認對話框。
2. **最新版本判定與雜湊檢測（Digest Change Detection，Not Chronological Order）**：
   - 「新版本」定義：僅指當前成功傳遞且內容通過驗證的固定檔案內容。
   - 透過內容 SHA-256 摘要（Digest）比對檢測檔案是否變更，**不承諾亦不依賴檔案修改時間 (`mtime`) 來保證全域時間順序**。
   - 自行匯出發布之相同 Digest 在讀取時予以忽略，不重複匯入。
3. **發布時機與髒旗標狀態機（Publication Timing & Dirty State Machine）**：
   - **觸發發布時機**：
     1. 本機新增／修改後的防手震批次處理（Debounced Batch）。
     2. 每小時定時檢查（Hourly Run）：**僅在本機處於髒狀態（Dirty）時才發布**；若本機無變更則絕不重複發布。
     3. 使用者明確觸發之手動發布操作（Explicit Manual Action）。
   - **發布中途編輯防護（Edit While Publishing）**：
     - 當背景正在發布版本 $N$ 時，若本機產生新的編輯 $N+1$，發布 $N$ 成功後**本機必須保持髒狀態（Remain Dirty）**，確保下一輪繼續發布 $N+1$。
     - 若發布失敗，本機同樣保留髒狀態（Retain Dirty）以待重試。
   - **下載與匯入永不標記髒狀態**：
     - 自同步資料夾下載或匯入外部合法快照後，**絕不將本機標記為髒（Download/import NEVER marks local dirty）**，避免無謂的反向回寫風暴。
4. **本機原子重新命名與雲端非端到端原子性防護**：
   - 本機產生的 `unified-lexicon.sqlite` 寫入臨時檔並完成驗證後，在本機以原子重新命名（rename）就位。
   - 但須注意：**本機 rename 在雲端同步軟體眼中並非端到端原子操作**。若下載時遇到不完整或中途同步檔案，系統**保留前一版已核准之本機快照**，待後續完整就位後再行讀取。
5. **AI 邊界聲明**：
   - AI 僅限於未來研究最佳發布時機（Publication Timing），**絕不允許 AI 自主核准或決定詞庫詞彙（Never approval of model words）**。

### 1.2 詞庫管理與候選規則
1. **中文人名與驗證讀音**：
   - 中文人名與已驗證之完整注音讀音（Chinese names + verified full readings）**預設作為中文人名候選字直接投影**。
2. **工程符號獨立映射**：
   - 既有的「中文完成文字 -> 工程符號/術語」映射機制（如 `H2O` -> `H₂O`、`SiO2` -> `SiO₂`）保持獨立運作，不與人名注音候選混淆。
3. **與學習/歷史分離**：
   - 底層動態學習庫（Windows `userphrase.sqlite`、Mac `learning.sqlite`）完全獨立分離，不放入共用快照中。

### 1.3 底層路徑與 libchewing (ec7f595) API 特性稽核
1. **使用者目錄與 CSV 來源雙重稽核路徑（Codex 程式碼查證）**：
   - 在 Windows 原生端，`isolated_editor` 將 `ChewingDev/chewing.dat` 作為使用者基準路徑傳入，libchewing 內部將其視為目錄並解析為 `ChewingDev/chewing.dat/v4/user_dict.csv`；
   - 而目前選單項目 `ID_HASHED` 則是開啟 `ChewingDev/v4/user_dict.csv`；
   - 未來實作與驗收時必須對**這兩個顯式來源進行獨立稽核與路徑校正**，絕不呼叫私有檔案。
2. **libchewing `ec7f595` API 行為**：
   - `UserDict::entries`：包含已被封鎖（banned）的詞條，且不包含 boost 加權資訊。
   - `unlearn_phrase`：單音節詞直接自詞典移除；多音節詞則是先加入黑名單（ban）再從歷史中移除。

---

## 2. 合成驗證矩陣 (Synthetic Verification Matrix)

> **重要標記**：先前之 CAS / 雲端 API 測試規劃已標記為廢棄（SUPERSEDED）。本矩陣針對使用者指定之同步資料夾全量快照模型訂定。所有測試項目狀態一律為 `NOT_RUN`。

| 編號 | 測試場景與分類 | 前置條件與測試輸入 | 預期系統行為與驗證標準 | 驗證狀態 |
| :--- | :--- | :--- | :--- | :---: |
| **M01** | **取消匯入預覽**<br>*(Canceled Preview)* | 載入包含 50 筆合成詞條之 SQLite 快照，於設定預覽視窗開啟後點選取消 | 1. 記憶體預覽完全釋放<br>2. 磁碟儲存層位元組未改變，無任何寫入<br>3. 輸入法候選快照保持原版本 | `NOT_RUN` |
| **M02** | **已知讀音 vs 歧義讀音**<br>*(Known vs Ambiguous Readings)* | 輸入中文人名「沈」：<br>- 條目 A 含已驗證讀音 `ㄕㄣˇ`<br>- 條目 B 具破音歧義（如 `ㄔㄣˊ` 未驗證） | 1. 已驗證讀音之條目 A 預設列入人名候選<br>2. 歧義未完整驗證之讀音不自動補齊或提權<br>3. 提示使用者手動核對讀音，不盲目猜測 | `NOT_RUN` |
| **M03** | **原生與共用排除分離**<br>*(Native/Shared Exclusion)* | 原生 libchewing 封鎖詞條「測試A」，共用 SQLite 獨立收納「測試A」 | 1. 共用詞庫之排除清單與原生 `user_dict` banned 分開管理<br>2. 遵守 `ec7f595` 特性，不因原生 banned 遺漏詞條或反向污染原生歷史 | `NOT_RUN` |
| **M04** | **完整組字 vs 部分組字**<br>*(Complete/Partial Composition)* | 正在鍵入注音音節 `ㄌㄟˇ`（未完成），此時同步資料夾偵測到新快照 | 1. 活躍組字緩衝區保持原狀，不中途刷新候選清單<br>2. 待使用者完成組字送出（Enter/空白）進入閒置邊界後，新快照才安全替換 | `NOT_RUN` |
| **M05** | **手勢按鍵不吞鍵防護**<br>*(F/Shift No Swallowing)* | 處於中文組字狀態下，觸發 F 分層符號手勢或 Shift 英文交接，同時存在未完成音節 | 1. `entering_syllable()` 嚴格防護，不吞噬當前按鍵<br>2. 狀態機平順切換，不遺漏任何按鍵事件，不因詞庫載入阻塞手勢 | `NOT_RUN` |
| **M06** | **過期/衝突副本忽略防護**<br>*(Conflict Copies Ignored)* | 同步資料夾中出現 `unified-lexicon (Conflict Copy).sqlite` 或舊副本 | 1. 系統嚴格只讀取 `unified-lexicon.sqlite`<br>2. 自動忽略任何衝突重新命名之檔案，不產生未預期的合併或崩潰 | `NOT_RUN` |
| **M07** | **不完整雲端檔案防護**<br>*(Partial Cloud File Integrity)* | 同步資料夾中 `unified-lexicon.sqlite` 處於同步軟體寫入中途（位元組截斷或損壞） | 1. SQLite 檔案完整性檢查（PRAGMA integrity_check）失敗<br>2. 拒絕載入並隔離該檔案<br>3. 核心維持既有合格記憶體快照並保留後續重試 | `NOT_RUN` |
| **M08** | **本地未送出覆蓋與髒狀態**<br>*(Dirty Local Overwrite & Retain)* | 本機正在發布版本 $N$ 時，使用者本機新增詞條進入版本 $N+1$ | 1. 版本 $N$ 發布成功後，本機狀態**依然維持 Dirty**<br>2. 下一輪排程能正確將版本 $N+1$ 輸出至固定檔案 | `NOT_RUN` |
| **M09** | **匯入不標記髒狀態**<br>*(Import Never Marks Dirty)* | 成功從同步資料夾載入並套用驗證合法之 `unified-lexicon.sqlite` | 1. 本機工作庫更新為新快照內容<br>2. **本機狀態明確保持非髒（Clean）**，定時排程絕不反向重複發布 | `NOT_RUN` |
| **M10** | **非法 Schema 架構拒絕**<br>*(Invalid Schema Defense)* | 嘗試載入缺少必要資料表、欄位名稱錯誤或包含非預期 trigger 之 SQLite | 1. Schema 結構檢驗即時失敗<br>2. 記錄安全審計錯誤，絕不執行非預期 SQL<br>3. 保留前一版合格快照 | `NOT_RUN` |
| **M11** | **拒絕信任檔案修改時間**<br>*(No Mtime Trust / Digest Only)* | 偽造本機或同步檔案之 `mtime`，但實體檔案內容之 SHA-256 Digest 完全未變 | 1. 系統透過 Digest 比對判定內容無變更，忽略 `mtime` 變更<br>2. 不觸發無效的重新載入或重複發布 | `NOT_RUN` |
| **M12** | **明確停用後防甦醒**<br>*(Explicit Disable No Revival)* | 使用者於設定中勾選停用共用詞庫，隨後重啟應用程式或系統服務 | 1. 記憶體投影保持為空，不查詢共用 SQLite<br>2. 絕不因偵測到磁碟快照存在而自動喚醒（Must not revive）舊資料 | `NOT_RUN` |
| **M13** | **發布失敗保留髒狀態**<br>*(Failed Publish Retains Dirty)* | 本機嘗試將快照寫入同步資料夾時遭遇磁碟滿或唯讀鎖定錯誤 | 1. 系統捕獲寫入異常，保持本機為 Dirty 狀態<br>2. 提示發布未完成，待下一週期或使用者操作時重新嘗試 | `NOT_RUN` |

---

## 3. 安全與執行邊界聲明
- 本文件僅為架構分析與合成矩陣規格規劃。
- **無私有資料存取**：絕不呼叫 API 存取使用者私有資料或 `%APPDATA%\Chewing`。
- **無環境變更**：未開啟任何 GUI/IME、未變更產品程式碼、未修改主要分支（`main`）、未執行安裝、資料遷移或服務重啟。
- 本矩陣全部項目維持 `NOT_RUN`，待 Codex 完成主規劃文件後對齊推進。
