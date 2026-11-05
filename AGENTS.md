# Chewing × Kanata × Context-Aware AI IME
# 代理不可破壞行為準則 (Agent Inviolable Rules)

> 文件代號：CKAI-AGENTS-001  
> 適用對象：所有參與本專案之 AI Agent 與人類開發者

---

## 1. 絕對不可破壞事項 (Strictly Inviolable)

1. **不可覆寫使用者原始 Kanata 配置**：
   - 基準檔案：[`config/kanata.original.kbd`](config/kanata.original.kbd) 與 [`docs/source/kanata.original.kbd`](docs/source/kanata.original.kbd)
   - 嚴禁變更該檔案內容；任何對鍵位之建議改動僅能以 `kanata.proposed.kbd` 呈現。
2. **不可破壞系統現存之正式新酷音輸入法**：
   - Windows TSF 模組必須使用獨立 GUID、獨立 ProgID 與 `%APPDATA%\ChewingDev` 目錄。
   - Linux Fcitx5 模組必須使用獨立 Addon 名稱與用戶目錄前綴 (`~/.local`)，嚴禁 `sudo make install` 覆蓋 `/usr`。
3. **不可破壞使用者個人詞庫與歷史資料**：
   - 絕不讀取、變更或覆寫 `%APPDATA%\Chewing\userphrase.sqlite` 或 Linux 既有個人詞庫。
4. **不可偽造驗證與測試結果**：
   - 未在實體環境實際執行編譯與測試之項目，狀態一律為 `NOT_RUN` 或 `BLOCKED`，嚴禁標註 `PASS`。
   - 正式測試日誌一律獨立存放於 `docs/verification/runs/`，不混入架構與研究文件。
5. **不可將未經授權之打字資料送往外部**：
   - 零雲端依賴；AI Reranker 預設關閉且僅允許本機 Loopback (`127.0.0.1:11434`)。
   - 密碼欄、安全桌面與敏感視窗一律停用 AI 與政策攔截。


## Windows 共用工作目錄

- 使用者指定 `C:\Users\Defu\Documents\gis-code\chewing-kanata-lab` 為 Codex 與 Agy 的共用 Windows 開發目錄；不再於 `ckai-codex` 或 `ckai-package-source` 啟動新的修改，那些目錄只保留歷史建置與驗證。
- 共用目錄中的分支切換、提交與同一檔案修改必須先協調；不得在另一代理編輯時切換工作樹。當前分工與來源見 `docs/development/SHARED_WINDOWS_WORKSPACE.md`。
- `upstream/windows-chewing-tsf` 與 `upstream/windows-chewing-preferences` 是獨立 Git 儲存庫；對齊各自 patch series，不要僅更新外層就宣稱完成同步。
- 原生設定頁含 F 延遲滑桿。舊 Explorer 載入舊 DLL 時仍可能開啟舊設定頁；先核對模組路徑，不要覆寫舊版套件或強制終止使用者應用。
