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
