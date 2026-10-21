# Windows TSF 模組安裝與隔離部署手冊

## 1. 隔離開發安裝 (無須提權，不影響系統正式新酷音)
1. 取得編譯完成之 `chewing_tip.dll`。
2. 開啟 PowerShell 執行隔離註冊腳本：
   ```powershell
   .\packaging\windows\register_dev_tsf.ps1 -DllPath ".\path\to\chewing_tip.dll"
   ```
3. 設定檔將自動讀取 `%APPDATA%\ChewingDev\policy.toml`。

## 2. 正式安裝包 (WiX / MSI 途徑)
- 透過 `cargo xtask package-installer` 產出之 MSI 安裝器進行安裝。
- 安裝精靈具備獨立顯示名稱：`新酷音輸入法 (ASCII-First Dev)`。

## 3. 解除安裝
- 執行 `.\packaging\windows\unregister_dev_tsf.ps1` 即刻乾淨清理登錄項。
