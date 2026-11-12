# Windows 開始列與 F 映射調整

更新時間：2026-11-12T13:44:51+08:00

## 使用者需求

- Zbook 的 1.1.4 在開始選單／工作列輸入框切回系統英文，Win+Space 無法重新選用 AI新酷音。
- 右下角移除官方網站、討論區、軟體回報與注音／選字回報；使用說明改連個人 GitHub。
- Restart 放最上方，移除首字母長按時間的子選單。
- 特殊符號／快捷符號頁改為可編輯的 F 分層映射表單。

## 範圍

新增映射功能依 X.Y.Z 規則使用 1.2.0-preview.search.1；開始列仍需使用者實機確認，因此有測試後綴。本輪不安裝、不重啟、不發布正式 Release、不合併 main。

AGY 負責選單與表單；Codex 負責安裝權限、共用預設表、設定儲存、原生輸入整合及驗收。

## 開始列診斷

WorkstationX 的實際註冊仍為 1.1.3，不能當成 Zbook 的實測。該安裝 DLL／公開字典只授予使用者、系統及管理員，沒有 All Application Packages 讀取 ACE；隔離 COM 登錄已有 AppContainer 讀取權限。1.1.4 安裝腳本有同一部署缺口。

Microsoft 說明 AppContainer 內的 IME 受到宿主限制；字典若不在 Program Files／Windows，須明確設定讀取 ACL：
https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements

新安裝流程在註冊前，先完整驗證套件，再僅對 manifest 列出的公開 payload 及其目錄加入非繼承的 ReadAndExecute。沒有授予寫入、沒有調整上層使用者資料夾、沒有修改私人詞庫或系統預設輸入來源。未知檔案不獲新存取權；Windows 可更新其 auto-inherited bookkeeping flag，但實際存取權不變。

這是已確認的部署缺口修正，尚不能宣稱已重現或解決 Zbook 的完整症狀。若新測試仍失敗，須取得該機載入／啟用錯誤再判斷，不能關閉系統防護或移除簽署要求。

## F 映射契約

同一份 Rust 預設表供原生查找及設定頁查詢。覆寫欄位 `f_layer_mappings` 儲存於獨立 ChewingDev 登錄鍵的 JSON，並隨既有設定 TOML 匯入／匯出。缺少欄位或無效登錄內容回到原預設。

缺少鍵代表預設，空字串代表原按鍵透傳，非空輸出限定一個非控制 Unicode 字元。F、修飾鍵與退格不開放重新映射；原組字／選字保護與 F 延遲保留。設定在沒有待處理組字時重新載入。舊 symbols.dat／swkb.dat 保留在既有設定載入儲存流程，移除頁籤不代表刪除原資料。
