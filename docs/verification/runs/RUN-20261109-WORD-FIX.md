# Word 修復複驗

時間：2026-11-09T23:20:51+08:00

原 AGY native 6c68e74 的審查問題已由 Codex 在 56a9321 修訂：

- OpenClose 僅接受明確 VT_I4 的 0/1；空值或錯誤保留原模式，不觸發錯誤提交。重複通知無額外切換。
- context 的未初始化 VT_EMPTY 允許原生流程；其他讀取失敗不授予可寫權限。
- 通知替換前結束舊 TSF UI 元件，停止計時與隱藏視窗；失焦、組字終止及停用也清理。英文組字每個字元不再建立一小時通知。
- 移除複製 match 的測試，改為呼叫正式轉換與 VARIANT 解析函式。

驗證：Windows `cargo test --workspace --locked --offline` 98 項通過（80+9+6+3），0 失敗。日誌 `%TEMP%\ckai-word-final-tests.log`。第 35 份補丁已加入 series，35 份從 pinned baseline 重播完成且 tree 一致。未執行安裝、重啟、main 合併、tag 或 Release。

限制：Word 實機、焦點切換、提示視窗、雙 Shift、F 分層與其他 Office 應用人工驗收仍 NOT_RUN。原 AGY 對 Office 停用 in-process AI reranker 與略過 imm32 patch 的策略仍保留，不能以單元測試宣稱所有 Office 功能正常。

交接：修復成果保存在 fix/word-openclose-and-shift-ux；外層回 main 前保留兩份舊未追蹤報告於 Documents/AI-Chewing/workspace-handoff-* 並核對 SHA256。原生 nested repository 回到與外層 main 契約一致的 release/v1.1.1，不切換至其歷史上游 main；修復分支完整保留。

後續獨立開發：使用者另授權 Shift 首字母長按轉小寫（按住字母鍵，不是 Shift）、本機統一 SQLite 與離線預覽核准匯入／匯出。尚無共同 DDL 定案，只有 term/representation/reading/binding/exclusion 邏輯提案；等待 Mac 負責者契約草案，Windows 不降版。本次 Word 修復不混入此功能；不改 Mac。

## 使用者授權重新安裝

時間：2026-11-09T23:28:08+08:00

已依使用者明確要求在 WorkstationX 完成 Release 建置、套件 preflight 與使用者層級安裝。套件 `C:\Users\Defu\Documents\AI-Chewing\1.1.2-preview.wordfix.1`，來源 outer 437ad79、native 56a9321、preferences cbc0cd3（沿用已驗證 1.1.1 設定程式及公開字典）。套件版本為測試識別；原生 PE 與設定程式內部版本仍為 1.1.1，未宣稱正式發布 1.1.2。

註冊 DLL：`C:\Users\Defu\AppData\Local\Programs\ChewingDev\versions\1.1.2-preview.wordfix.1-56a9321dd9a1\chewing_tip.dll`。已安裝與建置 DLL SHA256 均為 `55982cac81d75c9dfc75bcfd449ce3e7d7b31e9f9bc8ab152f89992f29871e04`；Installed apps DisplayVersion 為 1.1.2-preview.wordfix.1。保留舊安裝與詞庫，未終止 Word 或其他使用者應用；需要使用者重開 Word 載入新 DLL。Windows 共用外層維持乾淨 main。實機結果等待使用者回報。

## 使用者 Word 實機驗收

時間：2026-11-09T23:34:51+08:00

使用者回報：「測試後 Word 實作沒問題」。此回報確認本次已安裝 Word 修復在使用者測試情境下正常；不擴大解讀為其他 Office 應用、全部 F／Shift 邊界或 SQLite 新功能已驗收。Word 回報問題狀態：使用者實機驗收通過。
