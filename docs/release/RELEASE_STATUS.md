# 1.0.2 正式發布準備

使用者已於本輪確認 1.0.2-preview.2 功能正常，授權發布正式版。1.0.2 僅調整正式版本識別，新增的 F／Shift／共用候選需求另列 1.1.0。正式發布狀態須以 GitHub API 驗證紀錄為準。

# 前一已發布版本：v1.0.1

更新時間：2026-11-05T18:55:22+08:00

1.0.1 恢復原生完整設定頁、空白鍵選字、候選字型／大小與相關選項，保留 F 延遲滑桿。使用者已確認「設定頁與選字都正常」。

- 89 項無介面測試通過；前端與 Windows release build、workspace check、來源完整 tree 重現、套件雜湊與安裝檢查通過。
- 以同一獨立身分升級，安裝前後偏好相同。保留 1.0.0 檔案與發布附件。
- 右下角若仍顯示舊滑桿視窗，請儲存工作後登出再登入；已查證 Explorer 仍載入 .4 預覽版 DLL，不是 1.0.1 未安裝。
- 設定程式需要 WebView2。套件附完整來源、原版授權、manifest 與 SHA-256；未簽署。
- Ollama 原生整合、AI 糾正／預測與學習詞庫尚未完成，AI 維持關閉；獨立詞庫 GUI 編輯器不包含於本次。

[下載 v1.0.1](https://github.com/edwardhome/chewing-kanata-lab/releases/tag/v1.0.1)；[驗證與使用者回報](../verification/runs/RUN-20261105-NATIVE-PREFERENCES-04.md)；[共用 Windows 工作目錄](../development/SHARED_WINDOWS_WORKSPACE.md)。

前版：[v1.0.0 驗證報告](../verification/runs/RUN-20261105-V1-RELEASE-03.md)。

## 下一版：1.0.2-preview.2（開發驗證，尚未正式發布）

新增 Restart 恢復中文輸入、修正 Shift 殘留與組字期間的 editor 重建。自訂工程詞彙表單、附提示詞的批次文本及 GitHub 手動檢查／下載已實作。Ollama 傳輸與驗證核心已整合，TSF 的 AI 候選接線仍未完成。Zbook 由使用者自行安裝與操作，本輪未遠端安裝。

[本批測試及限制](../verification/runs/RUN-20261106-102-SETTINGS-TRANSPORT.md)。目前已發布版本仍是上方 1.0.1，不能因開發分支完成編譯就視為正式 1.0.2 已可下載。

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
