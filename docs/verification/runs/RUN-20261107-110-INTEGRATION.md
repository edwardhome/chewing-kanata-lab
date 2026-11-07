# 1.1.0 整合驗證（進行中）

更新時間：2026-11-07T17:39:30+08:00

## 已執行

- 原生固定來源 `e0986f9`，包含 AGY `03a40a4`、共用候選、context 綁定的提交佇列與版本 1.1.0。
- Windows 隔離來源 `%TEMP%\ckai-catalog-110-review\upstream\windows-chewing-tsf`；未安裝，未修改載入中的 DLL。
- `cargo test --locked --workspace`：65＋9＋6＋3＝83 passed，0 failed；xtask 與 doc-tests 無測試。日誌 `%TEMP%\ckai-session-110-native.log`，命令 exit 0。
- 兩項新增 production SessionControl 測試驗證：候選入列後排程被拒絕，恢復時只提交一次；文字已提交但結束排程失敗，恢復不再提交文字。
- 設定來源 `914f05f`：`npm run build` 通過，保留 Vite chunk 大小警告。

## 尚未完成的驗收

- AGY 新增的手勢 helper 當時只由測試呼叫；上述 83 項不能證明正式 F／Shift 路徑已完整驗證。正在改成正式輸入邏輯與測試共用防護，變更後需重跑。
- 最終 native／preferences Release、patch 重播、套件雜湊及 GitHub 正式發布：尚待完成。
- 實體 F 長按／短按、Shift 交接、未完成注音、候選跨頁、焦點切換及真實 TSF COM 失敗：NOT_RUN；不以無介面測試取代。
- Zbook 安裝與操作由使用者執行；本輪未遠端安裝。

## 後續修正重測

- 原生來源 `3279313`：移除測試專用的重複 handoff 邏輯，正式攔截／執行與測試共用防護；拒絕 F 分層的當前鍵繼續交給原生輸入流程。
- `cargo test --locked --workspace`：66＋9＋6＋3＝84 passed，0 failed，exit 0。日誌 `%TEMP%\ckai-final-110-native-tests.log`。
- 首次新增 guard 測試誤把既有 F＋Esc（反引號）映射視為透明鍵，測試失敗後依既有映射修正，改以透明 T 測試拒絕；未更動 Kanata 或原有映射。
- 設定來源 `914f05f`：Windows `cargo test --manifest-path src-tauri/Cargo.toml --locked --lib`，10 passed、0 failed、1 ignored（網路測試），日誌 `%TEMP%\ckai-110-prefs-final-tests.log`。
- 原生 29 個 patch 與設定 7 個 patch 完整重播；tree 分別精確匹配 `bd9d5f5feeac6e2e7ae7fed956b0096f99cf488b`、`43f044ab7a2d303443c915d692894b300a550e2b`。
- Release 建置進行中；以上仍不代表實體按鍵驗收或已發布。
