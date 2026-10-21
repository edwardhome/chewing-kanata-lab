# Antigravity 開發實作命令｜新酷音 × Kanata / Phase 1

> 文件類型：可直接執行的開發任務書（Implementation Order）  
> 日期：2026-10-22  
> 依據：`Chewing_Kanata_Development_Research.md`、`ANTIGRAVITY_START_HERE.md`、`kanata.original.kbd`  
> 工作重點：**從研究轉入實作，交付可編譯、可安裝到隔離開發環境、可關閉的最小功能**  
> 驗證分工：開發時照常編譯與執行自動測試；詳盡 QA、相容性、效能、VRAM、正式測試結論另立驗證文件。

## 0. 直接執行，不要只回傳方案

你現在是主要實作者（Antigravity）。**立刻開始實際開發，而不是再寫一份可行性分析或架構提案**。請閱讀三份輸入文件，以本任務書作為開工權限與優先級；若研究文件與實際上游程式碼衝突，以上游 checkout 的實況為準，並在開發紀錄註明差異。

先建立乾淨的開發工作區、新的 Git 分支與保存上游修訂資訊，接著依照 Phase 1 交付範圍進行小步修改、實際編譯及 commit。可以自行處理一般工程細節，不必每一小步詢問使用者。遇到有破壞現有輸入法／個人詞庫風險、需管理員權限安裝、或授權不明而需重分發時，停止該動作並清楚列出阻礙；繼續完成其他安全工作。

## 1. 使用者背景與問題（不得曲解）

- 使用者在 Windows 與 Linux 均使用**新酷音**，不是希望換掉新酷音。
- 使用者透過 Kanata 實現快速輸入 Markdown、程式碼符號與數字層；**現有 `kanata.original.kbd` 是不可修改的使用習慣基準**。
- 想在一般編輯、Markdown、程式碼中以 ASCII 半形標點為預設；正式中文書信與 Email 正文仍需要主動選擇中文標點、全形符號。
- 某些輸入框、應用程式會額外轉換輸入字元；請精確區分 Kanata Unicode 注入、輸入法按鍵事件、libchewing 組字、前端 commit 與應用程式事後轉換，**不可假定任何字元都經過 IME**。
- 後期將加入 Vim/Neovim Context Bridge，以及可由 Ollama 執行、預算低於 4GB 的本機 AI 候選字 reranker；本期不實作 AI。

## 2. Phase 1 必須實作的產品功能

### 2.1 最小模式政策

設計並實作一套小而明確、易於關閉的 `Character Output Policy`：

1. `legacy`：完全還原上游原生新酷音行為，**預設可作一鍵回退**；輸入法其餘功能及使用者詞庫不得改變。
2. `ascii_first`：對**經由本輸入法產生、且能安全判定為標點的輸出**優先保留 ASCII 符號。特別保護 Markdown 與程式碼慣用字元，例如 `[]{}():;<>`、反引號、單雙引號、反斜線、底線、管線、`#`、`*` 等。
3. `formal_zh`：保留可手動切換的中文標點選項，例如 `，。；：！？「」`；**不得把電子郵件地址、URL、程式碼內文全部變成全形英數字**。全形括號與其他標點需依上游已有行為或明確配置啟用，不可強制。

原則：同一鍵可能走原始鍵碼、keypad 或 `(unicode u+....)` 不同路徑。當事件已繞過 IME 或應用程式事後替換，僅記錄限制，不用全域 hook 或破壞其他應用程式的方式硬攔。

### 2.2 明確的安全邊界

- **只處理自身明確產生、尚未 commit 的標點**；不要對任意候選中文、整段文字、已提交字元執行通用字串替換。
- 保持完整 preedit/commit、候選字、撤銷與 focus lifecycle。對無把握辨識的情境保持上游原狀。
- 不使用全域鍵盤 hook、剪貼簿輪詢、常駐按鍵紀錄、應用程式螢幕截取或雲端傳送。
- 不允許 NFKC 一鍵正規化全部文字；這會破壞合法全形字元、相容字符與使用者意圖。
- 在 Vim Normal / Visual / Operator-pending / Command 模式，既有操作鍵必須不受影響。這期尚無可信外部模式訊號時，**不得靠猜 Esc / `:` / `i` 就自動切換模式**；只提供安全人工切換。
- 任何新功能可在不重裝系統、不清除個人資料的前提下停用。

### 2.3 配置與人工切換

- 先提供**最低風險的手動 profile 切換**（設定項、內建快捷鍵或前端現有設定介面，選一個最容易跨平台落實的方法）。
- 支援啟動時讀取配置；採用專案特定 namespace，別覆寫原有新酷音與 Kanata 配置。
- 未找到設定或解析失敗時回到 `legacy`，並只記錄非敏感、無輸入內容的診斷訊息。
- 明確區分 `default_profile`、`active_profile`、暫時覆寫；切換時不可遺失尚未提交的中文字。
- 如要新增快捷鍵，先查清和上游、新酷音、Kanata、桌面環境是否衝突；不得擅自改變原本 Caps、Ctrl、`f` hold 的肌肉記憶。

## 3. 修改位置與跨平台技術決策

### 3.1 上游工作樹（務必取得真實 commit ID）

- `https://codeberg.org/chewing/libchewing`
- `https://codeberg.org/chewing/windows-chewing-tsf`
- `https://github.com/fcitx/fcitx5-chewing`
- `https://codeberg.org/chewing/ibus-chewing`（暫列備援，非首輪實作）
- `https://github.com/jtroo/kanata`（只讀參考，**不得修改 Kanata 原檔**）

先依 README、原始碼與授權建立真實 source path map。不可將研究文件中的設計 API 當作上游已存在的函式。

### 3.2 實作切入原則

1. **adapter-first**：優先在 Fcitx5／Windows TSF 各自的事件與標點輸出接入層執行策略；如果跨平台可重用性需要，再抽成輕量、與 IME 平台無關的 policy 模組。
2. 只有確認平台前端無法保持一致時才改動 `libchewing`。不要重寫注音引擎、候選字語言模型與詞庫。
3. 跨平台共享政策應包含可描述的輸入／輸出與不變條件。Linux C++ 與 Windows Rust/C++ 可用薄 FFI/C ABI 或共用資料驅動規則；由實際版本決定，不要硬塞一套龐大 IPC 服務。
4. 先在一個平台跑通最小 Vertical Slice，再移植第二平台。**若當前開發主機只有其中一種 OS，就實作該平台、準備另一平台的 patch／CI 配置，不能假稱另一平台已實際編譯。**
5. Windows 必須尊重 TSF COM/Composition/TextStore 與 focus；Linux 尊重 Fcitx5 InputContext、preedit、commitString 與事件鏈。

## 4. 交付順序（不要停在純文字分析）

### Sprint A｜取得基線並開始修改

- 建立獨立工作區與 `feature/chewing-ascii-first-mvp` 或相當用途的新分支。
- 在 `UPSTREAM_INVENTORY.md` 記錄原始遠端、授權、鎖定的 commit、工具鏈與儲存庫關聯；保留 upstream history。
- 在 `CODE_PATH_MAP.md` 以實際檔案與函式標示注音／標點 KeyEvent -> libchewing -> frontend -> commit 走向，以及可能繞過 IME 的 Kanata Unicode 路徑。
- 寫出第一份**可執行**的 policy 模組，帶有 `legacy` / `ascii_first` / `formal_zh` 模式、白名單與不變條件；新增必要單元測試。
- 以一個可獨立 commit 的變更完成上述實作。

### Sprint B｜首選平台整合

- 以目前有真實可用工具鏈的平台為先（Linux Fcitx5 優先；Windows-only 主機就先 TSF），接上 policy 模組。
- 接上最小配置和 profile 切換；維持原版路徑和 feature flag 回退。
- 在開發隔離環境完成構建；不得直接覆寫系統正式安裝的新酷音。
- 提交可追蹤的獨立 commit，列出實際改動檔案。

### Sprint C｜第二平台與包裝

- 延伸到另一平台；明確說明和第一平台的差異。
- 如執行環境無法完成另一平台原生建置，仍建立原始碼 adapter、建置腳本／CI 定義及待環境執行清單，**不得把尚未執行的 build 標為成功**。
- 維持原始 Kanata 設定原樣，必要調整只能另附 `kanata.proposed.kbd` 並解釋差異，不能強迫使用。
- 製作開發版安裝／卸載及恢復原版說明，優先讓修改版與正式版共存。Windows TSF 若需要註冊不同 CLSID／IME identity，必須避免與正式版衝突。

### Sprint D｜為後期擴充預留介面，但不實作模型

- 預留 `EditorContext`（normal/insert/command/search/unknown + provenance），其值目前預設 `unknown`；不得靠鍵盤時間序列猜狀態。
- 預留可選的 `CandidateReranker` 契約（輸入候選與上下文，輸出合法候選排序），但此階段不啟動 Ollama、不下載模型、不在同步按鍵路徑加入網路請求。
- 若沒有可靠的 Neovim 訊號，只留 stub 與設計筆記；不要製造誤判。

## 5. 配置格式參考（**新設計草案，不是現有 API**）

可用 TOML/INI/JSON 之一；依實際前端既有風格選擇，命名與 schema 要留版本。

```toml
schema_version = 1
enabled = true
default_profile = "legacy"

[profiles.ascii_first]
mode = "ascii_first"

[profiles.formal_zh]
mode = "formal_zh"

[profiles.legacy]
mode = "legacy"

[editor]
mode_bridge = false

[ai]
enabled = false
```

從風險角度先讓 `legacy` 成為預設，以供最初開發期間安全回退；後續確認體驗後再決定是否將 `ascii_first` 設為預設。正式中文標點保持人為可選，不需識別 Email 軟體自動觸發。

## 6. 專案文件與程式碼交付清單

交付結果必須是**真實工作樹／可 review 的 patch**，不能只有新文件：

- 修改後的程式碼：最小 policy + 至少一平台 adapter 與配置接入。
- Git 分支、基準 SHA、每個實作 commit SHA、所有改動檔案與目的。
- `UPSTREAM_INVENTORY.md`：來源、授權、工具鏈基線。
- `CODE_PATH_MAP.md`：實際函式／檔案路徑、繞過邊界。
- `ARCHITECTURE_DECISIONS.md`：為何選擇改 adapter 或 engine，對應的替代方案。
- `CHANGELOG_DEV.md`：實際完成、待辦、已知風險、回退路線。
- `BUILD_AND_RUN.md`：可重現的安裝依賴、編譯、隔離啟用與卸載指令。
- `PATCH_PLAN.md`：後續 upstream sync、第二平台、Neovim／Ollama 接續。
- `docs/verification/README.md`：**只留正式驗證章節索引／待辦，不要把測試結果寫進研究文件**。

編譯及單元測試可以、也應該隨開發執行；具體紀錄寫在獨立 `docs/verification/`，包含命令、OS、工具版本、實際通過／失敗狀態。**不得聲稱沒有實測的平台、軟體或 GPU 已經通過驗證。**

## 7. 第一個完成點（可被檢查）

在完成 Sprint A 與 Sprint B 的第一平台 Vertical Slice 後，回報以下內容，再進入後續工作：

1. 目前 platform / OS / toolchain、upstream SHA 與 branch。
2. 最小 policy 的程式碼位置、模式設計、在哪個輸出節點生效。
3. `legacy` 如何精確回退原版行為。
4. 第一平台 adapter 的實際 code diff / commit SHA。
5. build 的實際執行情況和失敗阻礙（詳細 log 放獨立驗證檔案）。
6. 下次 commit 將處理的具體檔案與功能。

**沒有完成真實程式碼變更，不要回報「已完成 MVP」。**

## 8. 立即開始指令

現在請：

- 建立工作區與開發分支，固定上游版本。
- 完成必要 source inspection 後，直接寫出 policy 模組及其最小單元測試。
- 接上目前環境可工作的第一個 IME frontend，實際 build 並 commit。
- 以真實 diff、commit、build 狀態及下一步回報。

除非涉及不可逆動作或安全／授權問題，**不必再詢問是否開始；本任務書即為開始開發的授權**。
