# 驗證、相容性與測試報告索引 (Verification Hub)

> **目的**：依據專案規範，所有實際編譯驗證、單元測試、相容性測試、效能 Benchmark 與模型量測紀錄**全部獨立於本目錄**，不混入開發研究文件。

---

## 1. 驗證文件清單

| 文件名稱 | 說明 | 狀態 |
| :--- | :--- | :--- |
| [`01_UNIT_TEST_LOG.md`](01_UNIT_TEST_LOG.md) | `policy-core` (C-ABI) 與 `fcitx5-adapter` 單元測試實際編譯與執行紀錄 | **已通過 (32/32 C, 5/5 C++)** |
| `02_TSF_BUILD_LOG.md` | Windows TSF 模組編譯與打包紀錄 | 待執行 |
| `03_KANATA_COMPAT_MATRIX.md` | Kanata 原始鍵位 (`kanata.original.kbd`) 與 OS 注入實測矩陣 | 待執行 |
| `04_VIM_INTEGRATION_TEST.md` | Neovim ModeChanged 與 Normal 模式防誤觸實測 | 後續階段 |
| `05_AI_RERANK_BENCHMARK.md` | Ollama 本機模型記憶體與延遲實測報告 | 後續階段 |

---

## 2. 測試執行紀律聲明
- **嚴禁虛假宣告**：未在實體裝置或乾淨環境中執行的建置與測試，一律標註「待執行」，不得宣稱「已通過驗證」。
- **所有測試命令與環境可重現**：每次測試均詳實記錄 OS、編譯器版本、執行命令、輸入條件與終端原始輸出。
