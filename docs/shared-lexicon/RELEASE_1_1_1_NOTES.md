# Windows v1.1.1 版本發布說明 (Release Notes)

> **發布代號**：v1.1.1  
> **發布重點**：跨平台共用個人詞庫交換支援（Windows 與 Mac 基礎協議對齊）  
> **基準協議**：`ai-bopomofo.lexicon` / `schema_version: "1.0.0"`  
> **建置與打包負責**：由 Codex 獨立執行正式版本版號推進、建置、打包驗證與 GitHub 正式發布。

---

## 1. 主要新增功能與改進

### 1.1 跨平台共用個人詞庫交換 (Shared Lexicon v1)
- **對齊 1.0.0 交換規範**：
  支援基於 UTF-8 JSON 之 `ai-bopomofo.lexicon` 規範交換檔，支援 `zh_phrase`、`english_term`、`technical_alias` 與 `unicode_mapping` 四類詞條。
- **純記憶體預覽與本機明確授權 (Explicit Local Import Confirmation)**：
  匯入外部交換檔時僅於記憶體中預覽解析與差異比對（分類為新增、重複、衝突、不支援），在使用者明確確認前不寫入任何磁碟資料；取消操作保證不寫入任何變更。
- **獨立儲存層與版本回復 (Separate Store & Rollback)**：
  採用以校驗雜湊命名之不可變版本目錄與指標管理，完全不覆寫正式新酷音內部詞庫（`%APPDATA%\Chewing\userphrase.sqlite`）；提供隨時回復上一版功能，並於組字閒置邊界（idle boundary）平順生效。
- **中文詞候選字原生整合 (Native Chinese Candidates)**：
  匯入之中文詞條（`zh_phrase`）無縫投影至原生輸入法候選字清單，保持與原生既有詞庫相對排序的確定性。
- **精選私人工程詞匯出 (Selected Engineering Export)**：
  使用者可於設定介面手動勾選欲匯出的自訂工程詞，支援匯出為標準 v1 交換格式。

---

## 2. 邊界與已知限制 (Known Limitations)

1. **中文別名 (`zh_aliases`) 預設拒絕與明確略過機制**：
   - 因跨平台 v1 規範未納入中文別名，匯出包含 `zh_aliases` 之條目時系統**預設拒絕匯出**以防資料遺失。
   - 若使用者主動勾選略過別名，系統會清楚展示將被省略的別名清單並匯出標準 v1 檔案；本機原始自訂詞與其別名絕對完好保留。
2. **無自動中文私人詞庫匯出**：
   - 本版本僅支援手動選取之自訂工程詞匯出及共用交換快照管理，**不提供自動掃描或匯出正式新酷音內部中文私有詞庫**。
3. **容量與延遲量測聲明**：
   - 20,000 條（20k）邊界之耗時與延遲僅為除錯與開發階段之基準量測參考，**不作為正式執行階段之效能保證**。
4. **實機與實體按鍵驗收狀態**：
   - 實體鍵盤（Physical Keys）與 GUI 互動之端到端驗收狀態為 **`NOT_RUN`**，不以無介面單元測試替代實體驗收。

---

## 3. 測試與驗證數據 (Verification Status)

- **開發建置驗證 (Development Build Passing)**：
  - 開發建置階段通過 **270 項單元與整合測試**。
  - 通過 **23 項打包守衛（Package Guard）案例驗證**。
- **正式發布建置狀態 (Formal Release Build Pending)**：
  - 正式發布建置（Formal Release Build）之最終成品雜湊、PE 資源校驗與套件產出目前處於**待完成（PENDING）**狀態。
  - 嚴禁偽造未經實際產出之正式發布日誌與雜湊。
