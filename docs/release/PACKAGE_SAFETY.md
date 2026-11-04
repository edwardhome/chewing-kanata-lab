# Windows 套件隔離與交付條件

更新時間：2026-11-05T01:54:11+08:00

目前正在 `feature/windows-parity-codex` 修正交付流程，尚未生成符合條件的新 Windows 套件。

## 已確認的舊流程問題

舊 `do_install.ps1` 會將內附詞庫覆寫到 `%APPDATA%\Chewing`、把既有 F 延遲與 AI 偏好重設，並使用正式新酷音的 COM GUID。舊 `do_uninstall.ps1` 會刪除同一個正式 GUID 的註冊。Windows TSF 的舊 `tsfreg` 甚至會把不認識的參數當作解除註冊，因此不能執行舊工具的 `--identity` 等探測參數。

## 新流程

- 所有入口先檢查 `package-manifest.json`、固定產品身分、來源提交、版本、檔案路徑與 SHA-256。
- 執行任何註冊工具前，以檔案位元組確認 DLL 與 tsfreg 帶有隔離 GUID，且不帶正式新酷音 GUID。
- COM GUID 固定為 `{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}`；偏好與資料名稱固定為 `ChewingDev`。
- 僅修改 HKCU 的本產品 COM 註冊，使用獨立版本目錄；既有 DLL 不覆寫、不強制結束應用。
- 打包附帶的詞庫保存在該版本的 Dictionary 目錄，不部署到使用者私人詞庫。安裝器不重設偏好。
- 註冊後不執行 `-i`，保留目前輸入來源，使用者可用 Win+Space 選取。
- 解除安裝只操作經驗證的本產品安裝紀錄及獨立 GUID，保留詞庫、偏好與版本檔案。未有驗證紀錄的舊安裝不自動清除。

## Manifest 契約

必須由已稽核的原生建置交付流程產生，不能拿此格式替舊二進位背書。

- `schema_version`: 1
- `product`: `AI-Chewing`
- `clsid`: 上述隔離 GUID
- `registry_namespace`: `Software\ChewingDev`
- `data_directory`: `ChewingDev`
- `version`: 三段數字版號，可附開發版後綴
- `source_commit`, `windows_source_commit`: 外層與 Windows 來源的完整 40 位提交
- `files`: 相對檔名至小寫 SHA-256 的對照；必須包含 `chewing_tip.dll`、`tsfreg.exe`、`chewing.ico`。其他檔案僅允許根目錄檔名或 Dictionary 下的一層檔案。

`do_install.ps1 -ValidateOnly` 只做前置檢查，不安裝。測試命令為 `pwsh -File packaging/windows/test_package_guard.ps1`；這些檢查不啟動 DLL 或 TSF 工具。

## 發布前仍需完成

1. 對齊 DLL、tsfreg、偏好工具、詞庫載入與 IPC 的獨立身分及資料路徑。
2. 修正註冊工具對未知參數的處理，確實傳回註冊失敗。
3. 完成原生建置及 manifest 產出，再驗證安裝、升級、解除安裝與失敗回退。
4. 分別記錄自動測試與實際鍵盤／GUI 驗收。現有套件仍為 NOT_READY。
