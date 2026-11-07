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

## 發布套件核對

更新時間：2026-11-07T17:46:34+08:00

- 原生與設定 Release build：PASS，exit 0。保留既有 unused 與 LNK4104 警告；日誌 `%TEMP%\ckai-110-native-release.log`、`ckai-110-prefs-release.log`。
- 隔離建置來源與固定 Git 來源及前端 dist 共 196 個檔案 SHA-256 逐一一致。
- 固定套件來源：外層 `8c032496fecdddf7a9dd57855e7bcc5ac756fc9f`、原生 `32793133aaa10752da17eabde617f0da9a42f4c8`、設定 `914f05f5f5f6dae0129e04ff327b26963019108b`。
- 套件 25 個 ZIP 項目、24 個 payload 雜湊、三個 x64 PE、manifest 來源與 1.1.0 版本：PASS。Windows ZIP 字典路徑使用反斜線，核對時正規化分隔符並檢查重複與路徑穿越。
- `chewing_tip.dll` PE 版本 1.1.0.0；`ChewingPreferences.exe` 1.1.0；`tsfreg.exe` 沒有 PE 版本資源，以固定來源與雜湊識別，不宣稱三者都有版本資源。
- `do_install.ps1 -ValidateOnly`：PASS，未安裝或變更註冊。
- Windows ZIP 9,168,265 bytes，SHA-256 `4f590e930bb9d8b1e23583fffb3a16b72abaf32b3eff2ac496a4098215c83e7d`。
- 完整 Source ZIP 1,199,992 bytes，SHA-256 `eed6e53588c0117ac93e41039da358195f06dbc2ce9dc4936c143b56bc391542`；含三個固定 Git 來源與 SOURCE-COMMITS.json，無 .git、target、node_modules。
- 附件另含 package-manifest.json、SHA256SUMS.txt；Mac 副本 `/Users/defu/Documents/AI-Chewing/1.1.0/`，Windows 套件 `Documents\AI-Chewing\1.1.0`。
- 共用 Windows nested 來源已同步到上述固定提交，外層已同步 8c03249。兩份既有未追蹤歷史報告保留，不納入套件。
- 本節不宣稱 GitHub 已發布；實體按鍵／GUI 驗收仍為 NOT_RUN。

## GitHub 正式發布驗證

- PR #6 已合併；main 發布提交與 v1.1.0 tag 均為 `43bef989d67711beb697170606c275f08816dfbd`。
- Release ID `437480758`，發布時間 `2026-11-07T17:48:08+08:00`，draft=false、prerelease=false；Latest API 回傳 v1.1.0。
- 四個附件名稱、大小、GitHub SHA-256 digest 與本機驗證成品逐一一致。
- 正式網址：https://github.com/edwardhome/chewing-kanata-lab/releases/tag/v1.1.0 。本版未遠端安裝；實體按鍵驗收仍與自動測試分開。
