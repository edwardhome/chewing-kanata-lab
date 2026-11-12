# ZBook SearchHost COM 查找失敗：證據與候選修正

日期：2026-11-12（Asia/Taipei）。環境：ZBook-G11-FW，Windows 26H2 26300.9974，x64。
問題版本：`1.2.0-preview.search.2`。候選版本：`1.2.0-preview.search.3`，尚未安裝或驗收。

## 已確認的失敗環節

本次操作的第一個可觀測失敗是 **SearchHost 找不到新酷音 COM 類別**，在這段追蹤內尚未進入 DLL 載入。不能把它改寫為已測得的 Activate/HRESULT 錯誤，也不能由此宣稱後續所有初始化都會成功。

CLSID：`{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}`。

| 時間（+08:00） | 程序/PID | 操作、結果 | 原始匯出 CSV 行 |
| --- | --- | --- | --- |
| 17:22:56.9773625 | SearchHost.exe / 29112 | RegOpenKey `HKCR\CLSID\{CLSID}` → NAME NOT FOUND | 1052 |
| 17:22:56.9778056 | SearchHost.exe / 29112 | RegOpenKey `HKCR\WOW6432Node\CLSID\{CLSID}` → NAME NOT FOUND | 1053 |
| 17:22:56.9780571 | SearchHost.exe / 29112 | RegOpenKey `HKCR\PackagedCom\ClassIndex\{CLSID}` → NAME NOT FOUND | 1054 |
| 17:23:13.3771456 | Notepad.exe / 18864 | RegOpenKey 使用者 Classes silo 中相同 CLSID 的 InprocServer32 → SUCCESS | 7034 |
| 17:23:13.3773448 | Notepad.exe / 18864 | RegQueryValue 取得 `.2` 套件 DLL 路徑 → SUCCESS | 7047 |
| 17:23:13.3799203 | Notepad.exe / 18864 | Load Image `.2/chewing_tip.dll` → SUCCESS | 7175 |

表內 `{CLSID}` 是上列完整 GUID 的縮寫；完整原始路徑在交付的 `search-trace-relevant.csv`。`CsvLine` 指原始匯出檔行號，不是篩選後檔案的實體行號。

使用者儲存的實際檔名是 `search-trace.PML`，以 Microsoft 簽署的 Process Monitor 4.13 離線匯出 CSV，未重新開始擷取。原始檔 10,321 筆，期間 17:22:54.2014433–17:23:21.2121226。其中 9,817 筆是額外開啟的 Process Profiling；分析及精簡交付僅用剩餘 504 筆符合程序／產品路徑篩選的事件。SearchHost 在此範圍只有上述三筆註冊查詢。

證據 SHA256：

- PML：`be6674191bee65c1a8fb3c1f5e11cd1723bd165239c05d33abc2f7aa791ddfca`
- 原始匯出 CSV：`49e90a6a4efd1e0be45827eca32fa4977f51c456edb7f5fce994b273a0ccd5d8`
- 相關事件 CSV：`84cd65dab9a5629a2d32fbcd3d4165510033ae4cb080cdaacc4ff9ba7344addf`

## 交叉核對與推論界線

- 17:41:58 的唯讀診斷再次確認：原生 64 位元 HKCU COM 存在，HKLM COM 不存在，32 位元兩者都不存在。DLL 雜湊仍為 `6845687d61ffb9248bcbeaea7b9aafa42a5b2ca086a1a3209831952f49b6de3e`。目前安裝與 current.json 仍指向 `.2`。
- TSF 的 zh-TW 設定檔 `{0E8AD36A-67D0-41AA-96A1-448B80AAC427}` 已啟用。七項 category 與精確來源一致：KEYBOARD、DISPLAYATTRIBUTEPROVIDER、INPUTMODECOMPARTMENT、UIELEMENTENABLED、IMMERSIVESUPPORT、SYSTRAYSUPPORT、COMLESS。未註冊 SECUREMODE。
- 17:13 的程序檢查確認 SearchHost 為 x64 AppContainer，BinarySignaturePolicy flags=0。32 位元查詢是追蹤中存在的查找步驟，不能據此認定 SearchHost 是 32 位元或應安裝 x86 DLL。
- 以 SearchHost 實際權杖做唯讀 handle-open 檢查時，現有 DLL、圖示、六份公開字典都回 Win32 0；HKU 使用者 COM 路徑可開啟。直接開 HKU 與 SearchHost 自身 COM/HKCR 查找視圖不同，前者成功不能取代後者的失敗證據。
- 本機證據支持「目前只有 HKCU 的部署方式，在此 SearchHost 查找中未能解析類別」。不把這次測量推廣為所有 Windows/AppContainer 都不支援 HKCU COM 的定律。
- DLL 未簽章，但本次未見載入遭簽章政策拒絕的證據。現有事件紀錄未發現相關當機；兩者均不足以證明 Activate 成功。
- `Activate` 吞初始化錯誤、正常執行缺少可靠 logging 是後續可診斷性風險；本次沒有走到該階段的證據，故未修改。`TF_TMAE_SECUREMODE` 保護保留，未測得本次 ActivateEx 旗標。

## 候選修正範圍

新增原生 x64 HKLM COM 類別，並讓既有 HKCU 與 HKLM 的 InProcServer32 指向同一份公開 DLL，ThreadingModel 維持 Apartment。

機器層 COM 不可指向一般使用者能修改的 DLL，因此公開套件改放 `%ProgramFiles%\ChewingDev\versions\<version>-<native12>`。僅建立／保護產品專屬目錄；拒絕非信任 owner、非管理員寫入 ACL、junction/reparse 路徑和目錄外 DLL。先驗證既存父目錄，再複製 manifest 白名單檔案，最後核對雜湊與 ACL。無須放寬使用者上層目錄或私人資料權限。

HKLM 類別附安裝帳戶 SID 標記；拒絕覆寫其他帳戶或無標記的既有類別。COM 雙層寫入失敗時還原原值；移除器只刪除屬於目前安裝且 DLL 路徑吻合的類別。保留目前偏好、私人／共用／工程詞庫與版本檔案。仍需同帳戶 UAC。

修改檔案：`packaging/windows/{build_package,do_install,do_uninstall,diagnose_search,package_guard,test_package_guard}.ps1`；新增 `machine_registration.ps1`、`test_machine_registration.ps1`。`package_guard.ps1` 僅更新過時註解。診斷改列出 HKCU/HKLM 兩種位元視圖，預設輸出改到 TEMP。

不新增按鍵、組字或私人詞庫記錄。Native、Preferences、Kanata 及既有 41/20 份巢狀補丁完全不變。機器層部署會新增對這個隔離 CLSID 的機器可見性；其他 Windows 帳戶未驗證，安裝器不支援其他帳戶接管此安裝。

## 精確來源與測試

由原協作 Codex 交付完整 bundle，從 SynologyDrive 取得；未以 GitHub main 取代測試版。

- 來源交接 ZIP：6,325,656 bytes，SHA256 `a9349f7f04ee7d2605a79741b98bc6b9a88812b226513f79214ce524a999eede`。
- 外層基底 `2ca202c2bad7effab43422c076b70806eaa35caa`。
- Native `8374f3fca4ae579960291cb4816450ecdbd742cd`，tree `77c69a64e305bf345a6beea6721b6170ecf0f7cc`。
- Preferences `0701285cc94fb80c93041804d3c37b26d31fcba7`，tree `58d4bc1ce4f9030bd1b7ef8a83b7d50592020d5a`。
- 三個 repository 均核對 HEAD/tree，`git fsck --full --no-reflogs` 通過，使用各自隔離分支 `fix/zbook-search-com-20261112`。巢狀 repository 無程式變更。
- 候選套件重用已驗證 `.2` 的 DLL、tsfreg、Preferences exe 與六份公開字典，不宣稱重新編譯。另附封裝雜湊比對與補丁重播收據。

實際自動測試環境：Windows PowerShell **5.1.26100.9818**（64 位元）。

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File packaging/windows/test_package_guard.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File packaging/windows/test_machine_registration.ps1
```

| 項目 | 結果／範圍 |
| --- | --- |
| 套件驗證、身分隔離、公開 ACL、ValidateOnly、子程序錯誤及參數、腳本解析 | PASS，25 案例 |
| 機器部署 ACL／路徑、COM 同址、擁有者拒絕、重複註冊、部分寫入回復 | PASS，13 案例 |
| 測試日誌 | `20261112-zbook-search-com/package-guard.log`、`machine-registration.log` |
| 正式 HKLM 寫入、Program Files 部署、完整安裝／移除流程 | NOT_RUN，遵守不自行安裝限制 |
| Native／Preferences 重新編譯及其單元測試 | NOT_RUN，原始碼及交付二進位未改；本機無 Rust 編譯工具鏈 |
| Search/Word 等使用者流程的修正後驗收 | NOT_RUN |

COM 測試使用臨時 HKCU 兩個隔離分支模擬兩種註冊位置，真正執行 Registry API，未寫 HKLM 或產品註冊。檔案 ACL 拒絕案例直接測試安全描述元；在 COM 模擬案例中才替換受保護檔案前置檢查。這些測試不能代替正式提升權限的安裝測試。回復測試覆蓋 COM 值與既有 key ACL，不代表完整 TSF profile 的交易式回復已經驗收。

## 人工驗證與下一步

修正前使用者結果：Search 無法切回新酷音；回記事本可恢復，確認是搜尋欄局部失效。追蹤也確認記事本載入成功。Word 先前已由使用者驗證修復，本輪未重測。

使用者自行安排候選套件測試：

1. 解壓 `.3`，先執行 `do_install.ps1 -ValidateOnly`。準備好時才由使用者執行 `install_ime.bat`，同帳戶核准 UAC。安裝器不強制關閉或重啟使用者應用，不改系統預設輸入法。
2. 執行 `diagnose_search.bat`，確認 HKCU/HKLM 的 Registry64 都指向同一個 Program Files 版本及原 DLL 雜湊。保持 32 位元未註冊是本方案預期。
3. 記事本確認中文 → 開始／工作列搜尋 → Win+Space 選 AI新酷音並輸入固定測試文字 → 切英文再切回中文 → 回記事本 → 回 Word。回報每一步能否組字及送出，避免提供私人文字。
4. 補驗 F 分層與數字符號、組字中 Shift 英文切換、原生選字／空白選字、QWERTY 表單／單行頁籤／寬視窗，以及既有詞庫設定。
5. 若失敗，先保存同範圍的新追蹤。若類別仍 NAME NOT FOUND，繼續檢查註冊視圖；若類別成功且出現 Load Image，再調查相依資源／Activate。不要直接再安裝或移除 SECUREMODE 保護。

回退必須先用 `.3` 的移除器解除其擁有的機器層類別，再安裝保留的 `.2` 套件；不能只執行舊安裝器，否則 HKLM 可能殘留新路徑。此回退流程尚未在實機執行。版本檔案與私人資料都保留。

本次未安裝、未重啟使用者應用、未合併 main、未推送、未發布 Release。候選修正已針對直接證據，但整體搜尋輸入恢復仍待使用者驗收。

官方參考：[IME requirements](https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements)、[InprocServer32](https://learn.microsoft.com/en-us/windows/win32/com/inprocserver32)、[ActivateEx flags](https://learn.microsoft.com/en-us/windows/win32/api/msctf/nf-msctf-itftextinputprocessorex-activateex)。本案根因判定依本機追蹤，未將官方通用要求當成這台機器的錯誤證據。
