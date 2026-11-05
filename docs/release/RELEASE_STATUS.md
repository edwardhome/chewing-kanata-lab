# 目前發布版本：v1.0.0

更新時間：2026-11-05T17:50:35+08:00

使用者指定將已驗收的 Windows 修正版定為 v1.0.0。WorkstationX 已完成升級、實際解除安裝、確認註冊移除與重新安裝；現在顯示「AI新酷音」，不再使用預覽版名稱。

- 98 個無介面案例通過，0 失敗、0 跳過；release build 與 workspace check 通過。
- 十一份原生修補可重現相同完整 tree，DLL 檔案／產品版本為 1.0.0.0。
- Windows「已安裝的應用程式」加入 AI新酷音 1.0.0 解除安裝入口；保留偏好、使用者詞庫與版本備份。
- 同一獨立 GUID 升級，不建立另一個產品；其他 TSF 名稱／profile 清單與 ChewingDev 偏好比對未變。
- 使用者已在前版確認中文閒置 Enter／Backspace 與滑桿正常；本輪不修改這些行為。其他應用與完整實鍵驗收未完成。
- Ollama 原生整合、糾正／預測與學習詞庫尚未完成；AI 預設關閉。版號不代表這些功能已完成。
- 套件未簽署；發布附件包含 Windows 安裝包、完整原生與共用政策來源、manifest 與 SHA-256。

[下載 v1.0.0](https://github.com/edwardhome/chewing-kanata-lab/releases/tag/v1.0.0)；[本輪驗證報告](../verification/runs/RUN-20261105-V1-RELEASE-03.md)；[功能對齊與未完成範圍](../development/PARITY_MATRIX.md)。

前版 .4 的紀錄保留於 [回歸報告](../verification/runs/RUN-20261105-INPUT-SETTINGS-02.md)。版本備份保留是為了追溯與回復，不代表仍有另一個預覽版註冊。

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
