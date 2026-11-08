# 目前正式版本：v1.1.1

更新時間：2026-11-08T20:38:15+08:00

1.1.1 已正式發布，GitHub Latest 指向本版，非 draft／prerelease。

- 共用個人詞庫格式 1.0.0：預覽、本機核准、獨立儲存、回復及原生候選整合。
- 使用者選取工程詞匯出；中文別名預設拒絕有損轉換，明確同意省略才匯出基本欄位並保留原檔。
- 270 項離線測試、另 1 項 GitHub 實際查詢、23 項套件防護案例通過。正式 Release 重建、33/10 份 patch 重播、來源／公開字典／附件摘要核對通過。
- GUI／實體按鍵與真實安裝移除仍 NOT_RUN；未啟用背景輸入蒐集，私人中文詞庫自動匯出未做。

[下載 v1.1.1](https://github.com/edwardhome/chewing-kanata-lab/releases/tag/v1.1.1)；[完整發布驗證](../verification/runs/RUN-20261108-111-RELEASE.md)。

---

# 前一正式版本：v1.1.0

更新時間：2026-11-07T17:48:08+08:00

1.1.0 已正式發布，GitHub Latest 指向本版；不是 draft 或 prerelease。

- 注音已確認文字與 F 數字符號層／Shift 英文交接，保留未完成注音。
- 原生與工程候選共用選字視窗、分頁與來源動作；設定支援中文工程別名及舊格式備份升級。
- 有序的非同步提交佇列與 context 生命週期防護，區分接收文字與排程失敗。
- 原生 84 項、設定 10 項測試通過（另 1 項網路測試 ignored）；兩者 Release、來源重播與套件核對通過。
- 實體按鍵與應用 GUI 驗收仍為 NOT_RUN，發布不代表所有應用皆已實測；Zbook 由使用者自行安裝，未遠端安裝。
- Ollama 尚未接入 TSF 預測，背景輸入蒐集未啟用。

[下載 v1.1.0](https://github.com/edwardhome/chewing-kanata-lab/releases/tag/v1.1.0)；[完整驗證與限制](../verification/runs/RUN-20261107-110-INTEGRATION.md)。

---

# 前一正式版本：v1.0.2

更新時間：2026-11-06T17:56:04+08:00

使用者已確認 preview.2 目前功能正常，授權轉為 1.0.2 正式版。正式版只同步版本識別，不混入新的 1.1.0 行為。

- 自訂工程詞彙、附註解與大模型提示詞的批次文本、匯入預覽及匯出。
- 設定內 GitHub 手動檢查與下載，不自動安裝。
- 原生選單與設定中的 Restart：恢復中文輸入，保留未完成組字及修飾鍵手勢。
- 正式來源重跑 49 項測試，前端、原生及設定 Release 建置通過；完整來源 patch 重播、套件 24 個 payload 雜湊、三個 x64 PE 與安裝前檢查通過。
- Zbook 由使用者安裝與操作，本輪未遠端安裝。一般可用性由使用者回報，不代表所有邊界已逐項實測。
- 已知後續需求：注音組字時的 F 分層、Shift 英文交接、原生與工程共用候選，排入 1.1.0；本版不宣稱已完成。
- Ollama 尚未接入 TSF 的 AI 候選，AI 與背景輸入蒐集維持未啟用。

[正式下載](https://github.com/edwardhome/chewing-kanata-lab/releases/tag/v1.0.2)；[驗證紀錄](../verification/runs/RUN-20261106-STABLE-102.md)。

## 1.1.0 發布前整合紀錄（歷史）

- 原生與工程候選共用選字視窗、分頁及明確來源動作；中文工程別名及舊詞彙格式備份升級已實作。
- 非同步組字寫入改為保留有序提交佇列；寫入排程失敗與文字接收分開處理，避免重複提交。
- AGY 的 F／Shift 防護已合併，仍在補強實際輸入路徑與回歸測試；不得以測試專用邏輯宣稱實體按鍵驗收。
- 版本識別已準備為 1.1.0；最終 Release 建置、套件、來源重播及 GitHub 正式發布尚待完成。
- 本版仍不宣稱 Ollama 已接入 TSF 預測，沒有啟用背景輸入蒐集；Zbook 由使用者自行安裝與操作。

---

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

## 歷史開發階段：1.0.2-preview.2（已由正式版取代）

新增 Restart 恢復中文輸入、修正 Shift 殘留與組字期間的 editor 重建。自訂工程詞彙表單、附提示詞的批次文本及 GitHub 手動檢查／下載已實作。Ollama 傳輸與驗證核心已整合，TSF 的 AI 候選接線仍未完成。Zbook 由使用者自行安裝與操作，本輪未遠端安裝。

[本批測試及限制](../verification/runs/RUN-20261106-102-SETTINGS-TRANSPORT.md)。此段為正式版發布前的開發紀錄；目前正式下載以頁首 v1.0.2 為準。

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
