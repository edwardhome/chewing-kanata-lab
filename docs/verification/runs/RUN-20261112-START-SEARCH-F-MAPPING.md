# Windows 開始列、選單與 F 映射測試版

更新時間：2026-11-12T13:44:51+08:00

目標：1.2.0-preview.search.1。因新增可編輯 F 映射升次版本；開始列需 Zbook 實機確認，尚非正式發布。

## 變更與證據

- 已確認共同安裝缺口：使用者目錄中的公開 DLL／字典缺 AppContainer 讀取 ACE。加入完整套件驗證後的非繼承 ReadAndExecute；不授予寫入、不變更上層路徑與私人詞庫、不修改系統預設輸入法。WorkstationX 註冊仍為 1.1.3，因此不是 Zbook 1.1.4 的重現證明。
- AGY `8e0332c`：Restart 置頂，移除四項舊連結及長按時間子選單，Help 改為個人 GitHub。Codex 以 LoadLibraryEx data/image-resource 模式讀取已編譯 DLL 的選單，未啟用輸入法：第一項 ID=40029、四項舊 ID 不存在、只剩網路辭典子選單。
- AGY `43d7dd9`：兩個符號頁合併為 F 映射表單。Codex 加入後端共用映射、登錄儲存、TOML 往返、載入錯誤提示、輸入驗證及欄位標籤。
- Native `8374f3f`：41 份補丁完整重播 tree `77c69a64e305bf345a6beea6721b6170ecf0f7cc` 一致。
- Preferences `6fcac6e`：18 份補丁完整重播 tree `821bf55b06ad44763e8657b5308c89a598d9a975` 一致。

## 實際執行測試

- Windows locked/offline：核心 190、原生 109、設定 26，共 325 通過、1 項既有網路測試略過、0 失敗。
- 新增覆寫、留空、還原、控制鍵／控制字元拒絕、Unicode 單一字元、設定 TOML 往返與原生組字保護案例。
- 25 項套件防護測試通過，含權限不給寫入、保留原權限、反覆執行穩定、私人檔案權限及上層權限未擴張、損毀包不得變更權限。只使用臨時合成檔案。
- TypeScript/Vite 通過，2164 modules；現有大 chunk 警告保留。Windows 原生 Release 建置通過，未執行或載入為輸入法。
- 表單從後端取得同一份預設表；不重複手寫鍵位。輸出限制為單一非控制 Unicode 字元，空值為原鍵功能；F、退格及修飾鍵不可重新映射。

## 尚待實機驗證

Zbook 開始選單／工作列是否可選回並輸入中文、修改 F 映射後的實體按鍵、還原預設與 Word 回歸均 NOT_RUN。未安裝、不重啟使用者程式、不推送、不合併 main、不建立 Release。本測試版不能被描述為已修復全部開始列問題。

完整需求、權限修正依據及範圍見 [工作紀錄](../../tasks/START_SEARCH_F_MAPPING.md)。
