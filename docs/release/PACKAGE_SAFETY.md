# Windows 套件隔離與交付條件

更新時間：2026-11-05T03:03:30+08:00

## 已修正的舊流程

舊安裝器會覆寫正式新酷音的使用者詞庫、重設偏好，舊原生程式仍使用上游 GUID。舊 tsfreg 會把未知參數當作解除註冊，不能執行舊工具的探測參數。本次交付以完整修補系列與重新建置的程式為準，不能把新 manifest 套在舊二進位上。

## 新流程與邊界

- 固定 COM GUID `{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}`，ProgID `AI-Chewing.TextService`，偏好／使用者資料為 ChewingDev，獨立語言設定檔、顯示屬性 GUID 與命名管道。
- DLL／偏好工具從自身模組位置尋找資源。Editor 明確指定公開 Dictionary 與 ChewingDev 使用者字典；不搜尋或複製正式 Chewing 私人詞庫。
- 安裝前驗證 manifest 身分、兩個來源提交、版本、相對路徑、檔案摘要及二進位 GUID；拒絕路徑穿越、reparse point、缺漏資源、正式上游 GUID。
- 必需檔案包含 DLL、tsfreg、偏好工具、圖示與六個公開字典；建置腳本只複製明列檔案與授權聲明，完全不從使用者字典取資料。
- 安裝到 `%LOCALAPPDATA%\Programs\ChewingDev\versions\<version>-<nativeSHA12>`。同版本內容不同即拒絕；不覆寫已載入的舊 DLL，不強制結束應用。
- 安裝器明確寫入的 COM／ProgID 在 HKCU；Windows TSF RegisterProfile 自行處理獨立輸入設定檔的系統登錄，不宣稱全部 OS 寫入都只在 HKCU。
- 不呼叫 tsfreg 的輸入切換入口，不重設偏好、不安裝 Kanata，不改系統預設輸入來源。預覽版使用不同名稱供使用者辨認。
- 註冊工具未知參數先回傳錯誤；只有明確 -u 才解除註冊，錯誤會傳回安裝器。
- 安裝失敗會嘗試恢復原有本產品 DLL 指向；這是錯誤處理邏輯，不代表所有部分失敗情境已驗收。
- 解除安裝必須核對 current.json、版本套件與實際 DLL 登錄，僅操作本產品身分；保留版本檔案、偏好與使用者字典。未有驗證紀錄的舊版不自動清除。
- AI 預設關閉。安全啟用旗標與不可編輯情境有守衛，但密碼欄的實際 TSF 行為仍須人工驗收。

## 驗證與限制

`do_install.ps1 -ValidateOnly` 只做前置檢查。`test_package_guard.ps1` 使用合成套件、臨時目錄，不執行合成 EXE。位元組 GUID 與雜湊無法證明每個執行路徑安全，不能取代來源稽核與實鍵驗收。

實測與計數見 [本輪報告](../verification/runs/RUN-20261105-CODEX-PARITY-01.md)。升級、真正解除安裝、故障回退、Windows 應用／鍵盤相容性未執行的項目均維持 NOT_RUN。版本為未簽署開發預覽，尚非正式發布。
