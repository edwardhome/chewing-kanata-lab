# Windows 1.3.1 F 未使用手勢恢復

狀態：Windows 1.3.1 修正、編譯、測試與同步入口交付完成。未安裝；實體鍵盤驗收待使用者操作。

基準：正式 v1.3.0 的外層 tree 與 origin/main 0d7bee0 相同；native 0f49538、preferences 29a6361 均對應既有 patch manifest。新隔離分支 fix/f-idle-restore-1.3.1，Mac 工作樹 chewing-kanata-lab-f-idle-restore；Windows 使用 TEMP/ckai-f-idle-131 的獨立 clone，共用主線未切換。

已修正：以 buffer 的實際執行歷史決定是否恢復 F；append 有效非空符號才計為使用，刪空仍保留歷史。無效／空 append 與空退格不計為使用。未使用的正常放開或無映射普通鍵先恢復原字；取消／修飾鍵／焦點／模式失效不補字。

計時器沿用 generation 與原 TSF context/composition lifetime 檢查；不再以與訊息佇列綁定的 GetKeyboardState F 快照把一次性 timer 靜默丟掉。有效 tick 仍直接 render_empty_f_layer → set_composition_string → CompositionSession → RequestEditSession，更新標記組字，不送合成 Backspace。WM_TIMER 受主執行緒排程限制，不宣稱硬即時。

新增 GetMessageTime 同時基準差值檢查（處理 32-bit wrap）；若 timer 先到、實鍵訊息時間仍在門檻內且尚未執行分層命令，走普通打字。沒有將 Instant 與訊息時間相減。

實鍵驗收仍未做。現有新增計時器測試使用真實 Windows timer、測試接收者與 production SessionControl；不是宿主應用完整 TSF COM 編輯與可見文字的驗收。完整原生路徑另作程式追查，保留此限制。

交付已寫入 /Users/defu/Library/CloudStorage/SynologyDrive-Drive/Codex-Handoff/AI-Chewing/1.3.1。WorkstationX 的 D 槽不可用；不聲稱 Windows D:\SynologyDrive\Codex-Handoff\AI-Chewing 已下載。

不安裝、不重啟、不合併、不標籤、不發布。Ubuntu 與 v2 分支保留，Mac repo 唯讀。

## 最終程式驗證

- policy-core 全功能：220 通過。
- Windows native workspace：120 通過，包含 Windows 計時器／production queue，以及晚到訊息時間邊界案例。
- Windows preferences：28 通過，1 項既有網路測試略過。
- 前端 APP 規則契約：6 通過。
- 套件防護：25 通過；機器註冊隔離：13 通過（合成測試）。
- 合計 412 通過，1 略過；不重複計算重跑測試。
- TypeScript/Vite、native Release、preferences Release 均通過。現有 unused/linker PRIVATE 匯出提示保留，未擴張範圍清理。
- 原生 54 份／設定 30 份 patch 重播成功，與固定來源 tree 相同。
- 已在 Windows 實際產生 C:\Users\Defu\Documents\AI-Chewing\1.3.1，ValidateOnly 通過；DLL 與設定 EXE 的 FileVersion/ProductVersion 均為 1.3.1。沒有執行安裝。

## 來源

- 封裝外層來源：84372c3；核心修正：8619902。
- 原生：f53d4e9（包括 547f9e1）；設定：3c51f8e。
- 兩份 series.json 保存完整 commit/tree/hash。
- Windows 原始共享外層仍為乾淨 main；兩個 nested repositories 保持原本乾淨 feature/app-input-rules-1.3.0，沒有為製造乾淨狀態強制切換或丟棄檔案。
- 隔離 Windows 來源：%TEMP%\ckai-f-idle-131。日誌：%TEMP%\ckai-idle-{core-tests,native-tests,prefs-tests,native-release,prefs-release,package-tests,registration-tests}.log。

## 可重現驗證

在相同巢狀結構套用 manifest 補丁，於 policy-core 執行 cargo test --locked --offline --all-features；native 執行 cargo test --locked --offline --workspace 及 cargo build --release --workspace --locked --offline；preferences 執行 npm run build，src-tauri 執行 cargo test --locked --offline --lib 與 cargo build --release --locked --offline --bin ChewingPreferences。套件沿用 build_package.ps1 與 do_install.ps1 -ValidateOnly，不註冊輸入法。

訊息時間差依據 Microsoft GetMessageTime 契約（同時鐘毫秒值及溢位差值），https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagetime 。原生呼叫路徑以實際來源逐段核對；單元測試和 production queue 測試不代表已在 Word/Terminal 證明 TSF 可見畫面與實體按鍵行為。

## 最終交付核對

核對時間：2026-11-19T10:52:56+08:00

- 檔名：AI-Chewing-Windows-x64-1.3.1.zip
- 大小：11104964 bytes。
- SHA-256：`7f849c429af8ec1bf0baf508ebe4487b9c6f80199ed30793aa433e099451a473`
- ZIP CRC：PASS；29 個 manifest 內容檔案 SHA-256：全部 PASS；沒有未列入 manifest 的 payload。三份 PE（DLL、設定 EXE、tsfreg）：AMD64 PASS。
- 檔案已在 Windows 實際建立：C:\Users\Defu\Documents\AI-Chewing\AI-Chewing-Windows-x64-1.3.1.zip；解壓來源目錄 C:\Users\Defu\Documents\AI-Chewing\1.3.1。
- 已寫入並重新讀回核對：/Users/defu/Library/CloudStorage/SynologyDrive-Drive/Codex-Handoff/AI-Chewing/1.3.1/AI-Chewing-Windows-x64-1.3.1.zip。另附 SHA256SUMS.txt、測試說明.txt、本交接報告。
- Windows D:\SynologyDrive\Codex-Handoff\AI-Chewing\1.3.1 的雲端下載完成：**NOT_VERIFIED**。WorkstationX 的 D 槽不可存取，不以 Mac 同步入口寫入成功冒充 Windows 到檔。
- 首次 Compress-Archive 停滯，僅停止本次已識別的子程序；半成品保存於 Windows TEMP/ckai-idle-131-compress-archive.partial，未交付。改用 tar.exe -a 封裝，最終 ZIP 已獨立逐檔驗證。

最終程式／封裝來源提交（完整 SHA，報告後續提交不變更執行檔）：

- root：84372c3b62066f72de5167e35eea989747d33f3d
- native：f53d4e9578fb15d7c8fe082012d6798ff6ce2659
- preferences：3c51f8e6a7c53550ea15e7ed0bdf6e1b227d04f1

可供唯讀核對的實際完整測試 log 與 JSON 摘要，保存在本工作樹 ignored artifact：

`/Volumes/DATA/DATA/chewing-kanata-lab-f-idle-restore/.cache/verification/1.3.1/`

內有 core-tests、native-tests、prefs-tests、package-tests、registration-tests 五份 UTF-8 log，與 summary.json。均取自本次 Windows 執行結果，不是手工產生的 PASS 日誌。UI 的六項 node 測試另由本機實際執行，摘要計數已標示。

## 使用者驗收與發布授權

2026-11-19：使用者明確回報 Windows 測試完成、沒有問題，授權發布並上傳 1.3.1。驗收的是本報告列出的既有 ZIP，不重新編譯或替換其內容。Linux 驗收狀態不因本次 Windows 確認而改變。
