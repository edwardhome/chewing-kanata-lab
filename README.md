# 新酷音 × Kanata：跨平台 ASCII-first／全形標點相容輸入法研究

> **分支**：`research/upstream-audit`  
> **基準設定**：[`config/kanata.original.kbd`](config/kanata.original.kbd)（**不可修改之使用者基準**）  
> **專案原則**：優先既有開源架構、優先 Adapter-first、不從零開發輸入法、先確定性後 AI、零系統覆蓋風險。

---

## 1. 專案背景與目標

使用者長期在 Windows 與 Linux 上以 **新酷音 (libchewing)** 進行中文輸入，並搭配 **Kanata** 鍵盤映射工具進行鍵位重整。
目前在輸入 Markdown、程式碼及命令列時，`:`、`[`、`]`、`(`、`)`、`_` 等常用 ASCII 語法符號頻繁被新酷音強制轉為全形（如 `：`、`「`、`」`、`—`），必須反覆手動切換全形/半形。而在撰寫正式中文書信時，仍需標準繁體中文標點。

本專案之目標為：
1. **ASCII-First 標點優先**：在寫作 Markdown、程式碼、Shell 命令時預設輸出標準半形 ASCII 標點。
2. **全形標點無損相容**：在正式中文書信（`formal` Profile）或手動切換時，100% 保留原生新酷音中文全形標點。
3. **保留使用者肌肉記憶**：不修改使用者現有 `kanata.kbd` 配置（保持 `tap-hold`、`tap-dance`、Caps/Ctrl/Win+Space 映射）。
4. **跨平台核心小改**：以 Adapter-first 為優先，Windows (TSF) 與 Linux (Fcitx5) 前端共用輕量策略庫，不破壞上游注音核心。
5. **延後 AI 導入**：第一階段專注於符號政策與原始碼地圖，零雲端依賴，不引入神經網路模型。

---

## 2. 成果交付文件目錄 (`docs/`)

本階段依據規格已建立完整的開發研究與原始碼地圖文件（**不含任何實際編譯測試報告**）：

1. [**`docs/00_UPSTREAM_INVENTORY.md`**](docs/00_UPSTREAM_INVENTORY.md)  
   上游儲存庫清單、遠端來源、鎖定之 Commit SHA、版本號、SPDX 授權條款分析與建置工具清點。
2. [**`docs/01_CODE_PATH_MAP.md`**](docs/01_CODE_PATH_MAP.md)  
   跨平台原始碼路徑地圖：`KeyEvent -> libchewing -> preedit/commit -> application` 在 Windows TSF 與 Linux Fcitx5/IBus 的真實函式調用鏈、行號分析、libchewing 內部 `special_symbol_input` 強制轉換成全形的機制挖掘，以及 Kanata 注入與應用程式後處理邊界。
3. [**`docs/02_ARCHITECTURE_DECISIONS.md`**](docs/02_ARCHITECTURE_DECISIONS.md)  
   跨平台架構決策記錄 (ADR)：Adapter-first vs Core Extension 比較、共用 `policy-core` 設計、邊界劃分與安全回退。
4. [**`docs/03_PATCH_PLAN.md`**](docs/03_PATCH_PLAN.md)  
   最小修改規劃、Windows TSF / Linux Fcitx5 具體修改點、公開 API/ABI 影響評估、系統隔離與不安裝覆蓋防護、上游追隨與同步策略。
5. [**`docs/04_OUTPUT_POLICY.md`**](docs/04_OUTPUT_POLICY.md)  
   符號輸出政策詳細規格書：Profile 矩陣（`default`、`markdown`、`code`、`formal`、`raw-command`、`manual-fullwidth`）、受保護標記（URL/Email/Markdown/程式碼/Vim）、決策優先順序與快速鍵草案。
6. [**`docs/05_PLATFORM_WINDOWS.md`**](docs/05_PLATFORM_WINDOWS.md)  
   Windows TSF 內部架構深度研究：`ITfEditSession` (`InsertText` / `SetCompositionString`) 生命週期、按鍵過濾閘門與無痛 Bypass 機制。
7. [**`docs/06_PLATFORM_LINUX.md`**](docs/06_PLATFORM_LINUX.md)  
   Linux Fcitx5 / IBus 深度研究：`ChewingEngine::keyEvent` 分流、候選字攔截、Preedit 更新與 Flush 緩衝區沖刷。
8. [**`docs/07_KANATA_INTERACTION_ANALYSIS.md`**](docs/07_KANATA_INTERACTION_ANALYSIS.md)  
   Kanata 原始配置深度互動分析：小鍵盤事件流 vs Unicode 注入機制，以及為何 `(unicode u+003a)` 依然會被應用程式或輸入法變為全形的三大真實途徑剖析。
9. [**`docs/08_POLICY_CORE_DESIGN.md`**](docs/08_POLICY_CORE_DESIGN.md)  
   跨平台符號策略核心規格書：微秒級無鎖純演算法、查表規範（語法標點集與正式中文標點對照表）與 C-ABI 規格。

---

## 3. 設定、策略核心與 Patch 草案

* **使用者基準配置**：[`config/kanata.original.kbd`](config/kanata.original.kbd)
* **設定檔範例**：[`config/policy.example.toml`](config/policy.example.toml)
* **共用策略核心骨架**：[`policy-core/`](policy-core/)（純 Rust 實作與 C-ABI 標頭）
* **Windows TSF 最小補丁草案**：[`adapters/windows-tsf/adapter_tsf.patch`](adapters/windows-tsf/adapter_tsf.patch)
* **Linux Fcitx5 最小補丁草案**：[`adapters/fcitx5/adapter_fcitx5.patch`](adapters/fcitx5/adapter_fcitx5.patch)
* **上游工作樹同步腳本**：[`scripts/sync_upstream.ps1`](scripts/sync_upstream.ps1)

---

## 4. 上游版本基準速覽

- **libchewing** (`https://codeberg.org/chewing/libchewing.git`)：
  - SHA: `ec7f595ea7726fa06a9493e819b81669eba605a5` | 授權: `LGPL-2.1-or-later` | 版本: `0.14.0-alpha.4`
- **windows-chewing-tsf** (`https://codeberg.org/chewing/windows-chewing-tsf.git`)：
  - SHA: `053d31ba10c089ae98508dd4a908ac2c2be3cb49` | 授權: `GPL-3.0-or-later` | 版本: `26.9.0`
- **fcitx5-chewing** (`https://github.com/fcitx/fcitx5-chewing.git`)：
  - SHA: `60096e8ef49df21de6c155d7a0785a58fcccb955` | 授權: `LGPL-2.1-or-later` | 版本: `5.1.13`
- **ibus-chewing** (`https://codeberg.org/chewing/ibus-chewing.git`)：
  - SHA: `6e885f1a8556cb8cfcc657c5b0cfd71b93db4c5c` | 授權: `GPL-2.0-only` | 版本: `2.2.0`
- **kanata**：
  - 使用者基準: `config/kanata.original.kbd` | 授權: `LGPL-3.0-only`

---

## 5. 後續階段規劃 (非本階段範疇)

本階段嚴格遵守規範，不將實際編譯、驗收、模型實測混入本文件。後續階段將於**獨立的驗證文件**中展開：
- **Stage B**：在獨立開發環境驗證 Adapter-first patch 與 policy-core 符號放行。
- **Stage C**：Windows / Linux 獨立安裝器註冊與 GUI Profile 切換功能。
- **Stage D**：Neovim `ModeChanged` Context Bridge 實驗。
- **Stage E**：Ollama 本地候選字非同步重排序實測與 GPU VRAM 量測（另篇獨立驗證報告）。
