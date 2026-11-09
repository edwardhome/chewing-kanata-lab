# Windows 統一詞彙 SQLite 配合規劃

更新時間：2026-11-09T00:31:10+08:00

狀態：僅盤點與設計提案，未實作、未遷移私人檔案。研究分支：research/unified-lexicon-sqlite。

## 目標與範圍

以同一份平台中立的詞彙管理 SQLite 管理使用者中文詞、中文名稱、英文輸入碼、Unicode 輸出及其關聯；兩端 UI 對齊相同資料模型。近期先設計本機管理，不把本機索引或不可變安全快照視為另一份可獨立編輯的權威詞庫。最新方向優先離線手動匯入／匯出，先預覽再核准；雲端交換延後。

本輪不修改程式或 SQLite schema、不開啟私人詞庫、不安裝／重啟、不合併 main、不推送或發布。AI 飛輪、自動同步、背景輸入蒐集及 learning.sqlite 合併均延期。Mac repository 僅唯讀；Mac 對應文件為 docs/lexicon-learning/unified-sqlite-plan.md，應在兩邊文件完成後再共同凍結契約，不能將尚未看到的內容當成已對齊。

使用者已回報 Windows F 分層與 Shift 中文交接不再吞字；這是指定操作的人工確認，不是所有候選／SQLite 情境驗收。後續必須保留這個穩定基準。

## 已查證的 Windows 現況

來源基準：外層 e180b78、native c545fdb、preferences cbc0cd3。以下是原始碼盤點，沒有讀取任何使用者資料。

| 項目 | 實際來源／行為 | 規劃影響 |
| --- | --- | --- |
| 原生詞庫路徑 | crates/chewing_tip_core/src/shell.rs::isolated_editor 明確傳入 ChewingDev 的 chewing.dat 參數與公開 Dictionary 路徑 | 不能因參數叫 chewing.dat，就推定它是目前唯一的詞庫儲存檔 |
| libchewing 版本 | native Cargo.lock 固定 ec7f595ea7726fa06a9493e819b81669eba605a5 | API 判斷以該提交為準，不以另一個 checkout 的 HEAD 代替 |
| 原生實際初始化 | libchewing src/editor/mod.rs:234–269 會判斷 v3→v4 遷移、建立版本化目錄並讀取／初始化 user_dict.csv；也有獨立 history_dict.bin | 管理工具不能為「唯讀盤點」直接啟動實際 Editor::chewing，初始化本身可能寫入 |
| 私人工程詞 | policy-core/src/user_engineering.rs：engineering-terms.user.json，256 KiB 上限，與詞彙表驗證分開 | 未來是明確預覽、核准的遷移來源，不能直接覆寫或靜默截斷 |
| 中文工程別名 | engineering_lexicon.rs::chinese_matches 精確比對整段未送出中文，輸出 engineering outputs | 水→H₂O 是中文文字到符號映射，不是水的注音讀音 |
| 共用詞庫 | SharedStore 保存獨立 JSON 快照、版本與本機核准 receipt；TSF 背景 worker 產生 Arc<LexiconIndex> | 可作新 SQLite 的部署 Adapter 參考；編輯權威改為 SQLite 後，快照不得反向獨立編輯 |
| 現有失敗策略 | tip/src/text_service/shared_lexicon.rs::load 出錯回 Loaded::default，閒置時清除共用索引，原生候選繼續 | 新要求「保留上次核准快照」尚未實作，不能宣稱目前已符合 |

原生路徑另有來源碼差異：固定版本 libchewing 的 SearchPath 將 isolated_editor 傳入的 chewing.dat 視為資料夾，再接 v4/user_dict.csv；推導引擎路徑為 `%APPDATA%\ChewingDev\chewing.dat\v4\user_dict.csv`，但 ID_HASHED 選單開啟 `%APPDATA%\ChewingDev\v4\user_dict.csv`。這是原始碼推導，尚未讀取私人檔案確認；日後明確授權遷移時須分別盤點，不能自行搬移或任選其一。

### libchewing API 核對

已在 WorkstationX 的 Cargo 公開依賴原始碼 checkout 核對上述完整 commit；只讀程式，未呼叫 API 操作私人資料。

| 能力 | 固定版本 API | 限制與採用方式 |
| --- | --- | --- |
| 列舉 | Editor::user_dict() → &UserDict；UserDict::entries() → Iterator<(SyllableVec, String)>，src/editor/mod.rs:484、src/user/dict.rs:303 | entries 列出詞與讀音，但未帶 boost／排除旗標；其實作未過濾 banned，不能把每筆都當成啟用詞 |
| 新增 | Editor::learn_phrase(&mut self, &[Syllable], &str)；UserDict::insert(&self, …)，dict.rs:153 | 是引擎寫入能力，不是 SQLite transaction；管理詞初期建議走候選 Adapter，避免 SQLite 與 native 雙寫 |
| 刪除 | UserDict::remove，dict.rs:204 | 刪掉個人記錄不等於禁止內建詞，也不等於刪除統一管理詞 |
| 排除 | UserDict::ban、lookup_banned，dict.rs:217、235；Editor::unlearn_phrase，editor/mod.rs:494 | ban 使用 boost=MIN；lookup 排除該值。Editor::unlearn_phrase 的實作（editor/mod.rs:941–949）對單音節 remove、多音節 ban，並移除 history 記錄；需區分移除自訂詞、停用映射與禁止原生詞，不能共用一個模糊「刪除」動作 |
| 組字讀音 | Editor::symbols()，每個 Symbol::to_syllable() | 已有 full_composition_reading，拒絕未完成音節、混合符號及不符合字數的組字；目前支援完整組字 1–8 音節 |
| 文字反查讀音 | 本輪確認的公開 API 沒有證明任意中文文字可直接得到唯一正確讀音 | 不由字面、模型或中文別名猜測後自動啟用；可靠讀音要有來源與確認狀態 |

固定版本 UserDict::to_writer（src/user/dict.rs:122）輸出文字、讀音與 boost，可保留 banned 所需狀態；仍須獨立驗證匯入語意，不能單憑方法存在就操作私人檔案。

UserDict::entries 不包含完整學習語意，也不是自動學習歷史完整匯出 API。未來遷移若要保留 banned，必須以讀音精確查 lookup_banned 並解析對應文字，或使用經查證的保留狀態匯出方式；否則列為不支援，不能把排除詞重新啟用。直接呼叫 UserDict 變更後的持久化、引擎生命週期與多程序協調仍須另設合成測試，不能把方法存在等同安全遷移完成。

## 建議的平台中立資料模型

以下為邏輯欄位提案，不是已定案 DDL，名稱應再與 Mac 規劃對齊。

| 邏輯資料 | 用途與必要欄位 |
| --- | --- |
| term | 穩定 term_id、中文名稱／描述、來源、核准狀態、revision；一個概念可有多個名稱／輸出 |
| representation | term_id、精確文字、角色（中文名稱／英文碼／符號／一般文字）、排序；H2O、H₂O 分開保留，禁止用 NFKC 合併身份 |
| reading | 中文 representation_id、完整標準注音、聲調、來源、verified 狀態；多音讀音各自保存，缺讀音仍可管理但不得進注音索引 |
| binding | typed trigger、輸出 representation_id、action、啟用狀態；必須區分 phonetic_name、ascii_code、chinese_to_symbol |
| exclusion | 明確 scope（managed／native）、讀音、文字、原因及核准狀態；停用 managed 詞不隱含封鎖 native 同字詞 |
| catalog_meta / revision | schema 版本、權威來源 ID、資料 revision、遷移記錄與核准修訂；本機設定與核准 receipt 不等同雲端來源核准 |

每一種 binding 都有明確輸出，而不是把「中文別名」同時當作讀音與符號名稱。中文名稱預設輸出自身；例如 水＋ㄕㄨㄟˇ 的 phonetic_name 輸出「水」，既有 chinese_to_symbol（水→H₂O）仍須另外明確選取。英文碼 H2O 可關聯 H₂O，但不能因共用 term_id 而偷偷替換中文候選。

穩定管理 ID 與現有交換 entry_id 分開：交換 ID 仍依類型／讀音／輸出計算。資料關聯、排除範圍與 zh_aliases 無法無損放入 v1 JSON 時，明確拒絕或由使用者確認匯出可表示子集並列損失；本輪不修改 1.0.0 交換契約。

## Windows 候選 Adapter

管理介面在非按鍵執行緒讀寫 SQLite，交易完成後驗證核准 revision，產生不可變索引及可恢復快照。按鍵路徑只查記憶體，不開 SQLite、不等鎖、不讀雲端、不做模型請求。

沿用目前 CandidateSnapshot 的來源／動作識別與 generation、context、游標、讀音、原生候選一致性檢查；保留原生候選順序，新增名稱候選明確選取才提交。不能在 native 與 managed 有相同文字時丟掉動作身分，否則會把刪除、排除、送出範圍混用。既有 F／Shift handoff、pending commit queue 及原文優先不改。

現有 Shared 候選提交的是整段組字，不是任意游標附近的子詞替換。第一階段建議維持完整讀音／完整組字契約；若要求像原生詞庫一樣參與局部切詞、句中自動選詞，另列需求與 Adapter 設計，不能宣稱只換儲存層就完成。

SQLite 鎖定、損毀或 schema 不支援時：保留同一權威來源之上次核准快照，設定頁顯示失敗；無舊快照時只保留原生輸入。使用者明確停用或核准刪除則必須生效，不能當成讀取失敗而恢復舊詞。快照建立失敗時不切換 revision；交易提交與部署成功分開回報，允許重試部署。重啟後也要驗證快照來源、版本與摘要再恢復。

## 一份 SQLite 的精確邊界

統一的是「使用者可管理的詞彙內容」，不是所有引擎執行時資料。Mac learning.sqlite 暫不合併；Windows libchewing 的自動學習／history／session 資料也不在本輪吞併範圍。這些不應成為另一個可與管理 SQLite 競爭的人工編輯入口。

現有 native 手動新增／刪詞／記事本編輯與自動學習仍由舊引擎管理，不能直接宣稱已達單一管理權威。實作前須選擇：將手動新增／排除導向統一管理交易，或階段性顯示「舊引擎資料唯讀／待匯入」。遷移後不得仍讓兩個表單各自改 CSV 和 SQLite。是否將部分管理詞投影至 native engine 以參與斷詞，以及如何避免投影被重新當成使用者來源匯入，列為待決策。

## 最新方向：離線手動匯入／匯出

近期聚焦本機統一 SQLite、中文名稱／別名的可靠注音候選及統一管理 UI。使用者需在無法安裝雲端軟體的公司電腦操作，因此不依賴網路、雲端同步軟體或帳號。

匯出由使用者選擇位置，產出已核准內容的一致完整交換檔；匯入先驗證格式、完整性、容量、來源與讀音，呈現預覽、影響與無法表示的內容，使用者核准後才安全採用。取消或驗證失敗不變更現有詞庫；組字中不強制切換，按鍵路徑只讀不可變索引。不得把先前接受雲端衝突遺失解讀成手動匯入自動覆蓋授權。新格式的整份替換／逐筆處理仍待兩端共同定義，不提前實作。

既有 JSON 1.0 交換相容性保留；目前格式沒有完整別名與關聯欄位，不能宣稱無損交換全部新資料。完整新格式、識別、排除語意、遷移與預覽規則由兩端後續共同定義。

每週同步、固定雲端檔名、base SHA 仲裁及強制上傳均移至 [延後方案](deferred-cloud-sync.md)。本輪不啟用或實作它們。AGY 的同步矩陣保留為延後方案的設計資料，不代表已測試產品。

## 後續階段與驗收閘門

1. 共同定義 ID、名稱／讀音／符號關聯與排除語意，整理純合成 fixtures；確認離線匯入預覽與匯出相容性，不將傳輸綁入候選核心。
2. 另行授權實作後，先建立暫存 SQLite、交易／revision／唯讀快照與故障回退；測試中斷、鎖定、損毀、未知版本、保留上次核准、明確停用不可復活等情境。
3. 提供遷移預覽與精確衝突／排除／來源清單；私人實檔另行取得明確授權後才操作。保留原始備份、取消零寫入，先以合成 CSV／JSON 驗證，不直接打開正式新酷音詞庫。
4. 整合兩端管理 UI 與候選 Adapter，測試多音、異體、H2O/H₂O、相同輸出不同 action、未完成音節、焦點變化、快速 F／Shift、不重複提交及大詞庫查詢成本。
5. 使用者實機驗收後另定發布；AI 飛輪、learning.sqlite 合併與同步不隨此階段偷渡。

## 待決策

- 新完整交換格式與匯入預覽／核准規則；近期不要求同步資料夾或網路。
- 中文名稱只作明確候選（建議首階段），還是要參與 native 自動斷詞／局部選詞？
- 可靠讀音來源與人工確認 UI；多音詞如何呈現，不把結構合法視為發音正確。
- 手動 native 新增／排除導入 SQLite 的方式；舊 CSV 不再獨立編輯的切換點。
- 排除 scope、重複候選顯示、名稱與符號同列時的標示與快捷鍵。
- 有限本機恢復備份保留期與安全載入時機；手動匯入不可自動覆蓋。

## 本輪驗證狀態

完成原始碼盤點與文件檢查；未建立資料庫、未跑遷移、未呼叫私有詞列舉或寫入 API、未執行 GUI／實鍵測試。沒有新測試通過數；既有發布與使用者 F／Shift 回報僅是後續回歸基準。

## AGY 實際派工與審查紀錄

- 2026-11-09T00:23:56+08:00：檢查 WorkstationX 原 worker PID 19308 已不存在；舊終端 handle 7769 回 Unknown process，原對話 log 有 context canceled。背景 daemon PID 19628 仍在，不把它當作委派工作程序。
- 恢復原 Conversation ID `9718153a-085c-4fb6-bfa4-219555c0aaac` 的持續互動 CLI，確認新 worker PID 9032；未建立另一個專案對話或重啟 daemon。
- 已送出 `UNIFIED-SQLITE-PLAN-1109`，限定 AGY 只編寫 `docs/lexicon-learning/AGY_UNIFIED_SQLITE_MATRIX.md`，在隔離 research 分支提交；不得改來源、操作私人檔案、執行遷移或發布。
- 初始指令的條件更新 head 與衝突選擇已被後續使用者決策取代；已在同一終端送出固定 lexicon.sqlite、base_sha 比對、遠端變動放棄本機未發布修改的修訂，要求 download_no_echo／idle_no_publish／publish_during_edit／fixed_name_only／base_sha_conflict_cloud_wins。所有未執行項目須標 NOT_RUN。
- 已人工審查 AGY 提交 ba65561：三雜湊與雲端優先規則已納入，13 項合成案例均為 NOT_RUN。尚需修訂相同 hash 時的 generation 防護、衝突副本不得刪除、中文名稱與 ASCII 符號例子，以及完整 SQLite／WAL、發布失敗與競態案例。已回送同一工作階段；修訂未收回前不標示驗收完成。
- Mac 對應文件已唯讀確認每週最多一次、漏跑單次補跑、失敗等下週及手動同步。強制上傳最後補充已送同一 AGY 終端；待矩陣修訂收回後才能結束審查。

- 最新查收 AGY 1ae1a80 的 26 項規劃並人工審查；先前 generation、WAL 與同步順序問題已修訂，均 NOT_RUN。使用者其後改為離線匯入／匯出，該矩陣改列延後資料，不再作近期實作規格。
