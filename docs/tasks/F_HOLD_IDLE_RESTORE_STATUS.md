# Windows 1.3.1 F 未使用手勢恢復

狀態：修正已提交，第一輪 Windows 測試通過；事件時間邊界追加修正正在驗證，Release 編譯／封裝尚未完成。不可將此暫存進度當成交付完成。

基準：正式 v1.3.0 的外層 tree 與 origin/main 0d7bee0 相同；native 0f49538、preferences 29a6361 均對應既有 patch manifest。新隔離分支 fix/f-idle-restore-1.3.1，Mac 工作樹 chewing-kanata-lab-f-idle-restore；Windows 使用 TEMP/ckai-f-idle-131 的獨立 clone，共用主線未切換。

已修正：以 buffer 的實際執行歷史決定是否恢復 F；append 有效非空符號才計為使用，刪空仍保留歷史。無效／空 append 與空退格不計為使用。未使用的正常放開或無映射普通鍵先恢復原字；取消／修飾鍵／焦點／模式失效不補字。

計時器沿用 generation 與原 TSF context/composition lifetime 檢查；不再以與訊息佇列綁定的 GetKeyboardState F 快照把一次性 timer 靜默丟掉。有效 tick 仍直接 render_empty_f_layer → set_composition_string → CompositionSession → RequestEditSession，更新標記組字，不送合成 Backspace。WM_TIMER 受主執行緒排程限制，不宣稱硬即時。

新增 GetMessageTime 同時基準差值檢查（處理 32-bit wrap）；若 timer 先到、實鍵訊息時間仍在門檻內且尚未執行分層命令，走普通打字。沒有將 Instant 與訊息時間相減。

實鍵驗收仍未做。現有新增計時器測試使用真實 Windows timer、測試接收者與 production SessionControl；不是宿主應用完整 TSF COM 編輯與可見文字的驗收。完整原生路徑另作程式追查，保留此限制。

交付預計寫入 /Users/defu/Library/CloudStorage/SynologyDrive-Drive/Codex-Handoff/AI-Chewing/1.3.1。WorkstationX 的 D 槽不可用；不聲稱 Windows D:\SynologyDrive\Codex-Handoff\AI-Chewing 已下載。

不安裝、不重啟、不合併、不標籤、不發布。Ubuntu 與 v2 分支保留，Mac repo 唯讀。
