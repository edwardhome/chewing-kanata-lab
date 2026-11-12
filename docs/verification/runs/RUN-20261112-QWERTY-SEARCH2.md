# QWERTY 映射第二版驗證

更新時間：2026-11-12T16:32:37+08:00

版本 1.2.0-preview.search.2，未安裝或正式發布。

- 完成 QWERTY 映射、單鍵／全部還原；48 個後端支援鍵各出現一次，保護鍵不可修改。
- 設定視窗預設 1120 寬、可最大化；頁籤維持單行，窄視窗可橫向捲動。
- TypeScript / Vite 通過（2164 modules，既有大 chunk 警告）。Windows 設定測試 26 通過、1 項既有網路測試略過、0 失敗；Release 建置通過。
- 25 項套件防護測試通過。20 份設定補丁完整重播 tree 相同（舊 0011 補丁既有空白警告）。
- 唯讀 diagnose_search.bat / ps1 已在 WorkstationX 跑過；遮蔽家目錄與拒絕覆寫報告檢查通過。只收集版本、註冊、公開檔案權限、模組與特定當機 metadata，不讀取打字內容或私人詞庫、不變更系統。
- 原生 DLL / tsfreg 與 preview.search.1 位元組雜湊相同。本輪未修改原生行為、未重跑原生測試。
- ZIP CRC 與 28 個 payload 雜湊通過。ZIP SHA256：194f1150dbf204062f8008c5179c11e62599cf4e99122ed425375fd2049da3c3。
- 封裝來源 outer 2ca202c2bad7effab43422c076b70806eaa35caa、native 8374f3fca4ae579960291cb4816450ecdbd742cd、preferences 0701285cc94fb80c93041804d3c37b26d31fcba7。

## 未解決項目

使用者已確認 preview.search.1 在 Zbook 開始列搜尋仍不能輸入中文、切換後無法選回：FAIL。本版沒有修復宣告，仍需 Zbook 診斷資料。新 QWERTY 畫面與實體按鍵 GUI 驗證 NOT_RUN。

Windows 共用目錄保持乾淨 main，巢狀來源保持乾淨 release/1.1.4。編譯在 TEMP 獨立來源副本，未切換既有 AGY 工作樹；本輪沒有新建 AGY 程序，也沒有成功委派新工作。未安裝或重啟輸入法、未合併 main、未推送或建立 Release。
