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
