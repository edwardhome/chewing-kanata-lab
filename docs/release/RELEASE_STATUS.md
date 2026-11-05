# 目前發布狀態：PREVIEW_REGISTERED

更新時間：2026-11-05T14:23:04+08:00

Windows 獨立預覽版 `1.0.0-preview.20261105.4` 已完成建置、封裝、前置驗證與 WorkstationX 實際註冊。它不是正式穩定版，也尚未追平 AI-Bopomofo 0.6.14 全部功能。

- .4 修正中文無組字 Enter／Backspace 與設定滑桿重入死鎖；本輪 46 個案例通過；使用者已確認 Enter／Backspace 與滑桿兩項恢復正常，其餘實鍵／GUI 項目仍未驗收，詳見 [回歸報告](../verification/runs/RUN-20261105-INPUT-SETTINGS-02.md)。
- 前一輪 94 個無介面案例通過，0 失敗、0 跳過；四個交付元件 release build 與 workspace check 通過。
- 十份原生修補可從固定基準重現相同完整 tree；最終套件來源與檔案摘要可驗證。
- 名稱為「AI新酷音（預覽版）」；保留原有新酷音註冊、預設輸入法、語言清單、私人詞庫與 Kanata。未強制切換或重啟應用。
- 本輪重點為 F 組字／退格、Shift 英文編輯、工程詞庫候選、產品隔離及可靠安裝。實體鍵盤、密碼欄與應用相容性為 NOT_RUN。
- Ollama 原生整合與學習詞庫尚未完成，AI 預設關閉。不能將歷史 AI patch 名稱解讀為功能已可用。
- 套件未簽署。來源留在開發分支與 Draft PR，不合併 main；尚未建立公開 Release。

Windows 套件位於 `Documents\AI-Chewing\1.0.0-preview.20261105.4`；Mac 留有同版 Windows zip 備份於 `Documents/AI Chewing/1.0.0-preview.20261105.4/`，並非 Mac 安裝包。前兩個未交付候選包已撤出交付目錄。

- [驗證報告、版本與套件摘要](../verification/runs/RUN-20261105-CODEX-PARITY-01.md)
- [功能對齊與未完成範圍](../development/PARITY_MATRIX.md)
- [套件隔離與交付條件](PACKAGE_SAFETY.md)

## 歷史報告（不代表目前狀態）

以下原文保留作歷史紀錄；其中 RC-READY、AI 整合及跨平台宣告不能替代上方本輪驗證結果。

---

# 產品發布狀態報告 (Release Status)

- **產品名稱**：Chewing × Kanata × Context-Aware AI IME
- **版本門檻**：v1.0.0-rc.1 (Windows + Linux) / v1.1.0 Pre-Alpha (macOS)
- **Git 整合分支**：`feature/chewing-ascii-first-mvp`
- **最新 Commit SHA**：`a54d1ae` (後續 Commit 將更新)

---

## 1. 跨平台狀態矩陣

| 平台 | 原始碼狀態 | 建置狀態 (Build) | 測試狀態 (Tests) | 安裝與卸載 | 簽署狀態 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Windows 11 x64** | **DONE** (Policy + TSF Adapter) | **PASS** (chewing_tip.dll 4.1MB built) | **PASS** (32/32 Unit Tests) | 獨立 GUID 註冊就緒 | Unsigned (Dev Build) |
| **Linux Fcitx5** | **DONE** (Policy + C++ Bridge) | **PASS** (G++ Bridge Tests) | **PASS** (5/5 Unit Tests) | User-local 腳本就緒 | Open Source (GPL/LGPL) |
| **macOS Fcitx5** | **DONE** (Policy + Route Spec) | **NOT_RUN** (缺實體 macOS runner) | **NOT_RUN** | 打包腳本就緒 | Not Notarized |

---

## 2. 核心功能完成度

- [x] **F-001 (原版注音)**：100% 保留，零侵入核心。
- [x] **F-002 (legacy 回退)**：驗證通過，一鍵安全降級。
- [x] **F-003 (ascii_first)**：驗證通過，語法符號空緩衝區放行，Commit 轉回半形。
- [x] **F-004 (formal_zh)**：驗證通過，繁體全形標點完整對照保留。
- [x] **F-005 (保護 Email/URL)**：白名單機制已內建。
- [x] **F-007 (Vim-aware)**：Neovim Lua Plugin 實作完成，IPC 協定就緒。
- [x] **F-009 (Ollama Reranker)**：契約驗證與白名單過濾實作完成 (4/4 PASS)。

---

## 3. 發布判定 (Release Decision)

- **判定結果**：**RC-READY (Gate-A 已通過，進入 Gate-B 候選版)**
- **依據**：
  1. 所有核心純邏輯模組均已實際編譯並 100% 通過單元測試（32項 C-ABI + 5項 C++ Adapter + 4項 AI 契約）。
  2. 雙平台真實 Patch 與 Adapter Bridge 程式碼已落地。
  3. 未在 macOS 實機上運行的測試嚴格維持 `NOT_RUN`，無任何造假。
