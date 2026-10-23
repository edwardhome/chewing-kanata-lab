# Windows TSF 模組安裝與啟用指引 (Windows Installation Guide)

> **核心說明**：Windows 作業系統的輸入法架構 (Text Services Framework, TSF) 規定，任何協力廠商輸入法要在工作列語言清單（Win + Space）中出現，都必須透過 TSF Profile Manager (`tsfreg`) 註冊並呼叫系統 API `InstallLayoutOrTip`。此步驟需要彈出一次 UAC 管理員權限確認。

---

## 1. 一鍵快速安裝啟用 (推薦)

我們已經為您打包好了自動提權的一鍵安裝批次檔：

1. 開啟檔案總管，前往目錄：  
   [`dist\windows\`](file:///c:/Users/Defu/Documents/gis-code/chewing-kanata-lab/dist/windows/)
2. **雙擊執行 `install_ime.bat`**。
3. Windows 會彈出使用者帳戶控制 (UAC) 視窗，詢問是否允許變更，請點擊 **「是」**。
4. 批次檔將自動完成以下動作：
   - 將 `chewing_tip.dll` 與 `tsfreg.exe` 複製至專屬應用程式目錄。
   - 註冊 TSF COM 元件。
   - 註冊繁體中文 (0x0404) TSF Profile。
   - 呼叫 `InstallLayoutOrTip` 將新酷音正式加入目前使用者的語言切換清單。
   - 自動配置 `%APPDATA%\ChewingDev\policy.toml` 預設為 `ascii_first` 政策。

---

## 2. 立即體驗

安裝完成後：
1. 按下鍵盤快捷鍵 **Win + Space**（或您的 Kanata 原始 `@capsww` 快捷鍵）。
2. 在彈出的輸入法切換清單中，您將會看到 **「中文 (台灣) - 新酷音輸入法」**！
3. 切換選用後，開啟 VS Code、記事本或終端機：
   - 輸入 `:` 或 `[`，直接輸出半形 ASCII 符號。
   - 輸入注音符號，100% 保持原生新酷音中文組字與選字。

---

## 3. 一鍵完整卸載

若您日後想將此輸入法完全自系統中移除：
1. 雙擊執行目錄中的 [`dist\windows\uninstall_ime.bat`](file:///c:/Users/Defu/Documents/gis-code/chewing-kanata-lab/dist/windows/uninstall_ime.bat)。
2. 點擊「是」允許管理員權限。
3. 程式將自動從語言列清單中註銷，並清理登錄項，不留任何痕跡。
