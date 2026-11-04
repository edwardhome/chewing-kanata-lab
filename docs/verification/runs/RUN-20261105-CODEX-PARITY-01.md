# Windows 隔離預覽、F／英文組字與工程詞庫驗證

更新時間：2026-11-05T03:08:36+08:00

## 版本與執行範圍

- 外層來源：`e863e57b815d062fd3d53a28c32540c2ad74ab08`，`feature/windows-parity-codex`。
- Windows 原生來源：`85d61746c11f25f83db9c04826354e73a4398536`，`feature/codex-native-safety`。
- 原生基準：`053d31ba10c089ae98508dd4a908ac2c2be3cb49`；九份修補重現完整 tree `ba627a7f65306a57615cf8bce1ac2b664652741f`。
- 行為參考：AI-Bopomofo 0.6.14／`9700690321df88b35ddf987a034e6cf18a7fb3c1`。Mac 輸入法未安裝、未重啟。
- Windows runner：WorkstationX，MSVC Rust stable 1.99.1、Windows PowerShell 5.1。
- Mac runner：Apple clang 21.0.0、AddressSanitizer／UndefinedBehaviorSanitizer、可攜 PowerShell 7.5.4。
- 原本 Windows 工作目錄與未提交 AI 修改保留；本輪使用獨立工作目錄。Kanata 與正式 Chewing 使用者詞庫未讀取或修改。

## 實際測試計數

| 測試 | 通過 | 失敗 | 跳過 | 執行平台 |
| --- | ---: | ---: | ---: | --- |
| policy-core | 37 | 0 | 0 | Windows |
| chewing_tip | 13 | 0 | 0 | Windows |
| chewing_tip_core | 8 | 0 | 0 | Windows |
| tsfreg | 3 | 0 | 0 | Windows；最終顯示名稱修改後重跑 |
| package guard | 19 | 0 | 0 | Windows PS5.1；Mac PS7.5.4 也通過，相同案例不重複計數 |
| C AI reranker contract | 14 | 0 | 0 | Mac，ASan／UBSan |
| 合計 | 94 | 0 | 0 | 無 IME／GUI／音訊／自動打字測試 |

Rust doctest 共 0 個。C 契約測試不是 Ollama 推論測試。Rustfmt 元件不存在，格式化命令未通過；未冒稱已完成。

## 建置與來源重現

| 檢查 | 狀態 | 證據與限制 |
| --- | --- | --- |
| cargo check --workspace --locked | PASS | 包含 host／xtask；仍有既有 dead-code 警告 |
| 四個交付元件 release build | PASS | chewing_tip、chewing_tip_core、tsfreg、chewing_preferences；tsfreg 名稱調整後另重建 |
| 連結警告 | 已記錄 | LNK4104 DllGetClassObject export 提醒仍存在，不宣稱無警告 |
| 九份 patch 從固定基準重放 | PASS | 新 research 工作目錄的完整 tree 與 series.json 一致 |
| Windows Git checkout patch 位元組 | PASS | 九份 SHA-256 全符 |
| 實際建置的共用 policy 來源 | PASS | 16 個追蹤檔與套件來源一致；僅正規化 Windows checkout 的 CRLF 後比較 |
| 髒工作目錄拒絕 | PASS | -Check 非零結束，測試用未追蹤檔保持存在 |
| 舊 MSI build-installer／package-installer | PASS | 兩入口均明確拒絕，未產生 build/installer 或 dist |
| 真實 tsfreg 未知參數 | PASS | 最終工具啟動後實際回傳 2；在 COM 初始化前拒絕 |
| Windows 程式結束狀態 | PASS | 修正原本 & 呼叫 GUI-subsystem EXE 可能立即返回、讀到舊 LASTEXITCODE；改用 Process.WaitForExit／ExitCode，另驗證空白、引號與尾端反斜線參數 |
| 最終真實套件 ValidateOnly | PASS | manifest、必要字典、二進位隔離 GUID、摘要皆通過 |

## 最終候選包

- 版本：`1.0.0-preview.20261105.3`，未簽署開發預覽。原生上游 Cargo 版本仍為 26.9.0；交付版號由 package manifest 區分。
- 檔名：`AI-Chewing-Windows-x64-1.0.0-preview.20261105.3.zip`。
- SHA-256：`06404d3344c7fbd35ed9d2ca8a22fc7329e4d9a1d826455989204850775e5bc2`。
- 前兩個未發布候選包已撤出 Windows 交付目錄，保留在工作暫存目錄作追溯；不得使用。
- 公開字典：libchewing-data v2026.10.17，僅六個明列檔案。原始 Generic.zip SHA-256 `4e0ef8e9316db5ccea0aa0ebca98c70abbef3af5f493f254996736ce6cd7c8fb`。
- 保留原生 GPL、AI-Bopomofo MIT 與公開 v4 字典 CC BY 4.0／來源聲明。

## 操作驗收範圍

實際安裝／註冊為 PASS：最終套件已安裝至 ChewingDev 的獨立版本目錄，COM DLL 路徑、current.json 與 manifest 一致，繁中／簡中設定檔顯示預覽版名稱。解除安裝的 ValidateOnly 為 PASS，沒有執行真正解除安裝。

安裝前備份自身註冊與既有上游 GUID 登錄；安裝後逐一比較上游 HKCU／HKLM COM 與 HKLM TIP 匯出摘要，全數相同。預設輸入法 override 與使用者語言清單前後一致。保留之前自身 GUID 的 DLL 指向供追溯；未強制結束任何應用或切換輸入來源。這些證據是註冊層驗證，不能證明 DLL 在互動應用中已成功啟用。

真正解除安裝、既有已驗證版本升級與失敗回退為 NOT_RUN。密碼欄、候選視窗、切換輸入來源與實體鍵盤皆 NOT_RUN。

人工驗收清單：

1. 選取「AI新酷音（預覽版）」，確認傳統注音組字／選字與英文模式。
2. 短 F、短 Shift+F、直接英文及英文組字內 F+R；Shift+F+R 仍產生 +。
3. F 按住後多次 Backspace，不可刪到本次手勢之前的文字；放開 F 後原組字與游標合理。
4. 快速 F+U／M 的注音優先與長按後數字 8／2；含重複按鍵及修飾鍵。
5. Shift 英文游標移動、刪至空再繼續、Tab／上下候選、Esc 還原再取消、Enter／Space 明確接受。
6. 工程詞庫如 I2C→I²C，原文仍為第一候選；註解不進入送出文字。
7. 雙 Shift 僅乾淨短按切換；打字、修飾鍵、焦點與密碼欄不誤觸。
8. 記事本、瀏覽器與常用編輯器分開驗收；本輪未修改編輯器整合。

## 明確未完成

Ollama 原生模組尚未接入，AI 預設關閉；檢查時 loopback 服務未可用。700ms 去抖、2 秒間隔、單一推論槽、取消與過期結果生命週期、糾正／預測／前綴保留及學習詞庫均未完成。不存在 Windows fm 後端。這些差距不能用 C 契約通過或歷史 patch 標題宣稱已整合。

Antigravity 本機與 Windows 委派均曾嘗試；本輪受逾時／配額限制，未取得可驗收的新修改。歷史六份修補保留其來源；本輪新增修正與驗收由 Codex 完成。

## 重現命令

```powershell
cargo test --manifest-path policy-core/Cargo.toml --locked
powershell -NoProfile -File packaging/windows/test_package_guard.ps1
Push-Location upstream/windows-chewing-tsf
cargo test -p chewing_tip -p chewing_tip_core -p tsfreg --locked
cargo check --workspace --locked
cargo build -p chewing_tip -p chewing_tip_core -p tsfreg -p chewing_preferences --release --locked
Pop-Location
powershell -NoProfile -File scripts/apply_patches.ps1 -Check
```

獨立新工作目錄重現步驟見 [BOOTSTRAP_WINDOWS.md](../../development/BOOTSTRAP_WINDOWS.md)。原始測試與建置日誌保留於交付目錄的 verification 子目錄；沒有打字內容或憑證。
