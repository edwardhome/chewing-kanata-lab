# 1.1.0 共用候選接線檢查點

更新時間：2026-11-07T17:28:23+08:00

native commit `247f827`，基於 TSF 生命週期 `4d73af2`。

- 原生完整候選表與工程別名查找合併，保留原生順序及原始索引。
- Enter、設定選字鍵、數字別名共用 typed action；混合頁與 engine 頁分開並於原生選取前同步。
- 原生尾頁空白鍵 selector advancement 保留；工程項目不走 unlearn，註解只顯示。
- snapshot 比對世代、context、焦點、游標、注音與完整未提交組字；失效即關閉混合列表，避免走舊索引路徑。
- UI-less count/string/selection/page 與完整表一致，僅接受 canonical 頁界；空表與短 buffer 有防護。

隔離 Windows `cargo test --locked -p chewing_tip --lib` 62 通過、0 失敗。新增4項production paging helper測試；core CandidateSnapshot 4項測試已包含於119項核心測試。24個patch重播tree `532fcc0d8ae0b4a02394e8f0b21943dc1a17a7dc`一致。

審查已修正：native選取後空composition未結束、UI錯誤先ack造成遺失、數字別名誤選頁、snapshot失效仍留下mixed UI、跨context沿用候選。GUI／實體按鍵／真實無視窗TSF client尚未驗收，標記NOT_RUN，不把單元測試當作端到端行為證明。

AGY仍負責F／Shift unfinished-syllable與空F+Backspace防護；這批候選接線尚未同步到共享Windows來源，待兩邊提交完成再合併。1.1.0未封裝或發布。
