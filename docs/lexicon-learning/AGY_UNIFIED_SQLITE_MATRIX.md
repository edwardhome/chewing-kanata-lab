# 統一詞彙管理 SQLite 架構規劃與合成驗證矩陣

> **文件代號**：CKAI-UNIFIED-SQLITE-PLAN-1109  
> **分支**：`research/unified-lexicon-sqlite`  
> **狀態**：**規劃階段（Planning-Only），未實作／未執行（NOT_RUN）**。  
> **基準**：AI新酷音 Windows 1.1.1、pinned libchewing `ec7f595`、AI-Bopomofo Mac release。  
> **責任分工**：Codex 負責主規劃文件及整體架構審查；AGY 負責本規格規劃與合成測試矩陣。未經實機執行的驗收與測試項目狀態一律標註為 `NOT_RUN`。

---

## 1. 統一詞彙管理架構與職責劃分

### 1.1 統一詞庫管理（Vocabulary Management）與候選規則
1. **單一統一管理 SQLite 庫**：
   - 專門收納使用者明確管理之詞彙（含中文人名/專有名詞、已驗證完整注音讀音、自訂工程詞等）。
   - **中文人名與驗證讀音（Chinese names + verified full readings）**：預設直接投影為中文人名候選（Chinese-name candidates by default），以標準注音精確匹配。
   - **工程符號獨立映射（Chinese text -> engineering symbol mapping）**：既有的「中文完成文字 -> 工程符號/術語」映射（如 `H2O` -> `H₂O`、`SiO2` -> `SiO₂`）保持獨立機制，不與人名注音候選混淆。
2. **與動態學習／歷史完全隔離（Decoupling from Learning & History）**：
   - 輸入法核心動態學習、敲鍵選字頻率與歷史紀錄（如 Windows 之 `userphrase.sqlite`、Mac 之 `learning.sqlite`）**嚴格保持各自獨立，絕不合併入共用 SQLite 庫中**。
   - 依據專案不可破壞準則，不讀取、不竄改任何私有學習庫。
   - AI 重排序（AI Reranker / Phrase Prediction）維持遞延規劃（AI deferred），不依賴雲端。

### 1.2 雲端硬碟完整封閉快照同步機制（Cloud Snapshot Publishing）
1. **整體覆蓋原則（Whole Snapshot Replacement，No Row Merge）**：
   - 雲端硬碟僅存放完整封閉的 SQLite 快照檔案（Complete Closed SQLite Snapshot），**絕不進行資料庫行列層級的合併（No Row Merge）**。
   - 各端應用程式一律在**本機端獨立修改**，確認後將整份完整快照發布（Publish whole snapshot）至雲端。
   - 使用者明確知悉並接受整體覆蓋可能帶來的資料覆蓋遺失風險（User accepts possible loss from whole replacement）。
2. **版本仲裁與條件更新（Latest Definition & Concurrency Control）**：
   - **最新版定義**：以雲端端點成功的「條件式更新 / ETag 世代產生（Conditional update / ETag generation）」為準，**絕不信任本機時鐘或檔案修改時間 (`mtime`)**。
   - **並行發布競爭（Simultaneous Base N Publish）**：
     - 若兩端基於相同基礎版本 $N$ 同時嘗試發布，僅允許先滿足條件更新之一方勝出（成功更新至 $N+1$）。
     - 失敗的一方（Loser）**絕不盲目自動重試**，必須向使用者明確提示衝突，並提供雙選方案：
       1. **採用雲端整體版本 (Choose Cloud Whole Version)**：放棄本機變更，全量下載雲端快照覆蓋本機。
       2. **再次確認強制覆蓋 (Re-confirm Replace with Own Whole Version)**：由使用者主動授權，以本機完整版本強制推為雲端最新版本。
   - **無條件寫入提供者之退化策略**：若雲端儲存提供者缺乏條件寫入（Conditional Write）或 ETag 機制，系統強制退化為「單一發布者模式 (Single Publisher)」或在衝突時標註為「未解決衝突 (Explicit Unresolved Conflict)」，**嚴禁偽造全域順序（No fake total ordering）**。

### 1.3 核心輸入路徑安全與不可變記憶體快照
1. **熱路徑不可變記憶體原則（Key Path Immutable Memory Only）**：
   - 輸入法核心按鍵處理與候選字查詢路徑，僅存取**預先載入之不可變記憶體快照（Immutable Memory Projection）**，絕不於打字熱路徑中動態執行 SQLite 磁碟查詢或鎖定。
2. **損毀容錯與明確停用**：
   - 若本機 SQLite 檔案損毀或結構非法，系統維持並回退至**最後一次經本機明確核准的快照（Last locally approved snapshot）**。
   - 使用者明確「停用 (Explicit Disable)」詞庫時，系統保證立即卸載投影，且後續重啟時**絕不自動喚醒舊資料（Must not revive old data）**。

### 1.4 libchewing (ec7f595) API 邊界與行為對齊
- 依據鎖定之 libchewing 版本 `ec7f595` 實際 API 特性：
  - `user_dict().entries`：回傳包含 `reading` 與 `text`，但此列表**包含已被封鎖 (banned) 的詞條，且不包含權重加權資訊 (omits boost)**。
  - `unlearn_phrase`：若為單一音節詞，會直接自詞典中移除；若為多音節詞，其內部邏輯為先將該詞加入黑名單（ban），隨後自歷史中清除。
- 系統在與底層引擎介接時，必須精確考量此 API 行為差異，嚴禁對使用者私有資料直接呼叫破壞性 API。

---

## 2. 合成驗證矩陣 (Synthetic Verification Matrix)

> 所有測試項目均以合成資料（Synthetic Data）或獨立 mock 環境規劃。狀態一律為 `NOT_RUN`。

| 編號 | 測試場景與分類 | 前置條件與測試輸入 | 預期系統行為與驗證標準 | 驗證狀態 |
| :--- | :--- | :--- | :--- | :---: |
| **M01** | **取消匯入預覽**<br>*(Canceled Preview)* | 載入包含 50 筆合成詞條之 SQLite 快照，於設定預覽視窗開啟後點選取消 | 1. 記憶體預覽完全釋放<br>2. 磁碟儲存層位元組未改變，無任何寫入<br>3. 輸入法候選快照保持原版本 | `NOT_RUN` |
| **M02** | **已知讀音 vs 歧義讀音**<br>*(Known vs Ambiguous Readings)* | 輸入中文人名「沈」：<br>- 條目 A 含已驗證讀音 `ㄕㄣˇ`<br>- 條目 B 具破音歧義（如 `ㄔㄣˊ` 未驗證） | 1. 已驗證讀音之條目 A 預設列入人名候選<br>2. 歧義未完整驗證之讀音不自動補齊或提權<br>3. 提示使用者手動核對讀音，不盲目猜測 | `NOT_RUN` |
| **M03** | **原生與共用排除分離**<br>*(Native/Shared Exclusion)* | 原生 libchewing 封鎖詞條「測試A」，共用 SQLite 獨立收納「測試A」 | 1. 共用詞庫之排除清單與原生 `user_dict` banned 分開管理<br>2. 遵守 `ec7f595` 特性，不因原生 banned 遺漏詞條或反向污染原生歷史 | `NOT_RUN` |
| **M04** | **完整組字 vs 部分組字**<br>*(Complete/Partial Composition)* | 正在鍵入注音音節 `ㄌㄟˇ`（未完成），此時背景觸發詞庫全量快照更新 | 1. 活躍組字緩衝區保持原狀，不中途刷新候選清單<br>2. 待使用者完成組字送出（Enter/空白）進入閒置邊界後，新快照才安全替換 | `NOT_RUN` |
| **M05** | **手勢按鍵不吞鍵防護**<br>*(F/Shift No Swallowing)* | 處於中文組字狀態下，觸發 F 分層符號手勢或 Shift 英文交接，同時存在未完成音節 | 1. `entering_syllable()` 嚴格防護，不吞噬當前按鍵<br>2. 狀態機平順切換，不遺漏任何按鍵事件，不因詞庫載入阻塞手勢 | `NOT_RUN` |
| **M06** | **過期雲端快照拒絕**<br>*(Stale Snapshot Rejection)* | 雲端快照之 ETag / 條件版本為 $N-1$，本機已發布至版本 $N$ | 1. 系統偵測到過期快照，拒絕自動套用<br>2. 阻止以舊版雲端檔案覆蓋較新本機快照 | `NOT_RUN` |
| **M07** | **不完整雲端檔案防護**<br>*(Partial Cloud File Integrity)* | 雲端下載之 SQLite 檔案處於傳輸中途（位元組截斷或 header 損壞） | 1. SQLite 檔案完整性檢查（PRAGMA integrity_check）失敗<br>2. 拒絕載入並隔離該檔案<br>3. 核心維持既有合格記憶體快照 | `NOT_RUN` |
| **M08** | **髒本機全量覆蓋裁決**<br>*(Dirty Local Whole Overwrite)* | 本機有未發布之修改，同時雲端已被另一台裝置發布新版 $N+1$ | 1. 系統明確偵測版本衝突<br>2. 阻斷盲目背景發布<br>3. 彈出明確對話框供使用者選擇「全量載入雲端版」或「以本機全量版覆蓋」 | `NOT_RUN` |
| **M09** | **回復作為全新修訂版**<br>*(Rollback as New Revision)* | 使用者選取回復至歷史快照 $R_{old}$ | 1. 回復不以刪除歷史方式進行，而是將 $R_{old}$ 內容作為全新修訂版本 $N+1$ 儲存與發布<br>2. 確保雲端條件更新具遞增單調性 | `NOT_RUN` |
| **M10** | **非法 Schema 架構拒絕**<br>*(Invalid Schema Defense)* | 嘗試載入缺少必要資料表、欄位名稱錯誤或包含非預期 trigger 之 SQLite | 1. Schema 結構檢驗即時失敗<br>2. 記錄安全審計錯誤，絕不執行非預期 SQL<br>3. 保留前一版合格快照 | `NOT_RUN` |
| **M11** | **拒絕信任本機檔案時間**<br>*(No Mtime Trust)* | 偽造本機檔案之 `mtime` 使其晚於雲端，但雲端 ETag / 條件版本較新 | 1. 系統比對雲端 ETag / 版本識別，判定雲端為最新版<br>2. 忽略任何篡改之檔案時間戳記，防止倒退覆蓋 | `NOT_RUN` |
| **M12** | **明確停用後防甦醒**<br>*(Explicit Disable No Revival)* | 使用者於設定中勾選停用共用詞庫，隨後重啟應用程式或系統服務 | 1. 記憶體投影保持為空，不查詢共用 SQLite<br>2. 絕不因偵測到磁碟快照存在而自動喚醒（Must not revive）舊資料 | `NOT_RUN` |

---

## 3. 安全與執行邊界聲明
- 本文件僅為架構分析與合成矩陣規格規劃。
- **無私有資料存取**：絕不呼叫 API 存取使用者私有資料或 `%APPDATA%\Chewing`。
- **無環境變更**：未開啟任何 GUI/IME、未變更產品程式碼、未修改主要分支（`main`）、未執行安裝、資料遷移或服務重啟。
- 本矩陣全部項目維持 `NOT_RUN`，待 Codex 完成主架構審核後推進。
