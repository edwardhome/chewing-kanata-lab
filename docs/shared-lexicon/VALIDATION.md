# 共用詞庫首批驗證

日期：2026-11-08。開發分支：feature/shared-lexicon-v1。

## Mac 實際解析器

直接編譯 AI-Bopomofo 97006903 的 Packages/LexiconLearning/Sources/LearningCore/LexiconSchema.swift 與 tests/shared-lexicon/check_mac.swift，執行合成 valid-v1.json：PASS，exit 0。

7 個條目涵蓋中文注音、H2O 原碼與 H₂O、SiO₂、CH₄、FinFET／finfet 身分差異。Python hashlib 產生獨立身分 oracle，Swift 實際驗證 ID，並完成 decode/export/decode 的逐欄相等比較。沒有啟動 GUI、安裝輸入法或讀取私人詞庫。

## GIS 有界分析

Run ID：20261108T171125-72eb59d4；模型 gis/GIS-AI-MODEL，Thinking off；117.3 秒。輸入只含公開 schema、驗證器與 Windows 工程詞程式副本；僅允許產生 compatibility.md。

控制器 PASSED，execution exit 0，event_errors 空，verifier exit 0，source_unchanged=true，scope_violations／pre_verification_scope_violations／missing_required_changes 均空。

**Codex 人工結論：不接受該報告作為實作規格。** 格式及來源完整性驗收不能證明分析正確。雖然確實指出中文別名、大小寫身分與容量差異，但有以下問題：

- 建議丟棄別名或塞進 annotation，違反無損交換目標。
- 把 Windows 工程詞的資料結構推論為整個 UI 沒有核准流程；未提供設定頁來源，結論超出證據。
- 中文測試將「立場」配成錯誤的注音；不能採為詞庫 fixture。
- 將 JSON Schema 的字串 maxLength 與實際 Swift UTF-16 上限混為一談。
- 說大量合併會增加 JSON 深度，沒有結構依據；應分開驗證條目數、位元組數及深度。
- 多處行號與格式／平台標示不精確，不作可執行契約引用。

原始 result.json、changes.diff、verify-0.log 與工作副本保留在 OpenCode runs/20261108T171125-72eb59d4；沒有把原始 PASSED 改寫成失敗，也沒有套用模型輸出到產品。CONTRACT.md 由 Codex 對照實際程式獨立整理。本次未量測 token 節省率。

## Windows 與雙向交換驗證

更新時間：2026-11-08T18:55:08+08:00

在 WorkstationX 隔離暫存樹執行，沒有安裝、重新啟動輸入法或操作私人詞庫：

- policy-core：138 單元測試、5 預測測試、4 工程匯出測試、9 共用詞庫測試，共 156 通過。
- 原生 TSF workspace：76 + 9 + 6 + 3，共 94 通過。
- 設定後端：20 通過、1 略過。新增 10 項涵蓋選取、取消、來源過期、別名省略確認、既有檔案防覆寫、鎖定及匯入衝突處理。
- 合計 270 通過、1 略過、0 失敗。略過案例未宣稱驗收。
- 設定前端 npm run build 通過；保留既有的大型 chunk 警告。
- AGY 提供 8 組非法 JSON，均由 Rust 測試確認拒絕。

雙向測試使用真實 Windows Rust 解析器與 Mac LexiconSchema.swift：valid-v1.json → Windows encode → Mac decode/encode → Windows decode/encode。最後以 Python 比較所有 JSON 欄位，7 筆條目與來源、審核、使用統計等元資料完全相等。這是格式交換證據，不是兩端 GUI 操作證據。

20,000 筆合成案例保留全量索引與候選，單次 debug 測得索引建立 39.2573 ms、查找加候選快照 18.9028 ms；不是效能保證，仍需實機輸入延遲評估。

## 待完成與限制

開發實作已包括獨立儲存、差異預覽、本機確認、校驗、原子部署與回復、原生候選整合，以及使用者選取工程詞後匯出。巢狀儲存庫完整 patch 重播已通過（原生 30、設定 8，tree 相等）；尚未完成版本封裝與實機 GUI／按鍵驗收，沒有發布本功能；現行 1.1.0 不宣稱具備雙平台共用詞庫。

v1 不接受 zh_aliases。預設拒絕有損工程詞匯出，只有使用者明確允許省略、檢視確切省略清單並再次確認，才產生基本欄位交換檔；原始工程詞不變。私人中文詞庫自動匯出未實作。

背景載入以閒置邊界切換快照；TSF worker 成功啟動後 DLL 保持載入至宿主程式結束。未進行 GUI、實體鍵盤或已安裝輸入法驗收。
