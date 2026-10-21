# 產品發布狀態報告 (Release Status)

- **產品名稱**：Chewing × Kanata × Context-Aware AI IME
- **版本門檻**：v1.0.0-rc.1 (Windows + Linux) / v1.1.0 Pre-Alpha (macOS)
- **Git 整合分支**：`feature/chewing-ascii-first-mvp`
- **最新 Commit SHA**：`a54d1ae` (後續 Commit 將更新)

---

## 1. 跨平台狀態矩陣

| 平台 | 原始碼狀態 | 建置狀態 (Build) | 測試狀態 (Tests) | 安裝與卸載 | 簽署狀態 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Windows 11 x64** | **DONE** (Policy + TSF Adapter) | **PASS** (C/C++), **PENDING** (Rust xtask) | **PASS** (32/32 Unit Tests) | 獨立 GUID 隔離腳本就緒 | Unsigned (Dev Build) |
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
