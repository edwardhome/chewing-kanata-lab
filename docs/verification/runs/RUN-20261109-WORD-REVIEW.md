# Word 修復獨立審查

更新時間：2026-11-09T23:13:05+08:00

來源：外層 246b9d1；native 6c68e7403165c777724681ece5f1fd159ee65bba；對照 c545fdb。結論：尚不接受發布，需要修訂與 Word 人工驗收。

## 實際驗證

- WorkstationX 隔離 clone：`C:\Users\Defu\AppData\Local\Temp\ckai-word-review-5929c4c15b934a58bae7147d171b61b2`。
- `scripts/apply_patches.ps1 -Check`：來源樹及補丁雜湊檢查成功。
- `cargo test --workspace --locked --offline`：96 項通過（78 + 9 + 6 + 3）、0 失敗，exit 0；完整輸出保存於上述目錄 native-tests.log。首次執行因 PowerShell 把 cargo stderr 視為錯誤而中止，修正驗證腳本後重新執行成功；不是產品編譯失敗。
- 從 pinned baseline 在另一隔離 clone 重播 34 份補丁：全部完成，tree 與 series.json 完全一致。
- 未安裝、未重啟、未操作 Word；Word／Office 實機與提示視窗生命週期驗收 NOT_RUN。

## 阻擋問題

1. **高優先：Shift 提示生命週期。** chewing.rs:852 起每次 SessionResult::Updated 呼叫 show_message，建立一小時提示。show_message:1770–1799 每次 Notification::new 呼叫 BeginUIElement，直接覆寫 self.notification，沒有先 EndUIElement。notification.rs:379、385–393 顯示註冊與解除須配對；未找到 Drop 自動解除。新高頻路徑可留下多個長時間 UI 元件。應重用單一提示或在替換前完整結束舊元件，並覆蓋失焦／組字終止清理；需人工視窗驗收。
2. **高優先：讀取失敗不能當關閉。** chewing.rs:1258–1274 使用 unwrap_or(0)，把 COM 失敗或 VT_EMPTY 轉成明確關閉；on_compartment_change:1333 起可能因此停用中文並提交組字。應區分有效 0/1、未初始化與錯誤；失敗不改模式，不觸發提交。既有 is_context_mutable 同時忽略所有 compartment 錯誤而非僅 VT_EMPTY，也需收斂成明確可接受狀態。
3. **驗證缺口：新增兩項測試複製 match。** chewing.rs:2764 起並未呼叫 production transition，production 回歸仍可能全綠。將轉換抽成正式程式共用函式並測試有效狀態、重複通知與未知值；COM、Word 焦點與實鍵另行驗收。
4. **相容性範圍：Office 全面停用 AI reranker。** ai_client.rs 將 Word/Excel/PowerPoint/Outlook 全數列入禁用；quirk.rs 同時擴大略過 imm32 patch。這是實際功能範圍變更，尚無逐一程式驗證證據，不以 Word 修復測試代表全部 Office 已驗收。

## 詞庫規劃

已收回 AGY 1ae1a80，保留作者歷史合併至研究分支；26 項皆為 NOT_RUN。generation、WAL、每週嘗試與強制上傳修訂已人工檢查。最新使用者改為離線手動匯入／匯出；雲端規則改列 deferred-cloud-sync.md，矩陣明示延後，不視為近期產品實作。

## 交接限制

共用 Windows 外層與 native 仍在 fix/word-openclose-and-shift-ux；保留兩份既有未追蹤報告。未確認其他 AGY 已停止寫入，不擅自切 main。舊持續終端 handle 68013 已失效，嘗試回傳審查未送達；未建立新 AGY 對話或重複派工。需確認目前工作程序／對話後再回傳修訂要求。
