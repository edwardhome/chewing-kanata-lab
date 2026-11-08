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

## 未完成

Windows 共用 JSON 解析／持久化、中文詞候選適配、匯入差異預覽與回復、Mac 中文別名擴充、雙向端到端驗收與發布仍未完成。現行 1.1.0 不宣稱已具備雙平台共用詞庫。
