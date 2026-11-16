# 1.3.0-dev.1 使用者實測與後續驗收

更新時間：2026-11-16T13:15:38+08:00

## 使用者實測

- Notepad 與檔案總管的 App 切換正常。
- Windows Terminal 的 App 切換無反應。規則路徑為 `c:\program files\windowsapps\microsoft.windowsterminal_1.24.12741.0_x64__8wekyb3d8bbwe\windowsterminal.exe`。
- 使用者認為本輪 App 功能整體測試成功；這不等同 Terminal 或 F parity 已通過。
- 安裝由使用者操作；已授權請 Zbook 既有 Codex 對話協助唯讀診斷。

## 確認的 F 契約

按下 F 時立即呈現原本輸入（標準注音的 ㄑ 或英文 f），保持可撤回預覽。只有超過設定的長按時間才撤回本次 F 預覽並等待分層。短按、期限內下一鍵、既有組字與 Shift+F 都必須維持正常輸入，不能刪除其他文字或用系統 Backspace 撤回。

1.3.0-dev.1 刻意沿用 1.1.5 的 F 實作，尚未交付上述 parity。現行程式在 F keydown 只存事件，短按放開才 replay，因此使用者回報與程式一致。

## 已查收但未接受的 AGY 候選

WorkstationX root a75a7d0、native 8ec6227；均保存在 feature/app-input-rules-1.3.0。僅取得 Git bundle 供本機唯讀審查，沒有覆蓋共用工作樹、安裝或發佈。

native 候選的阻擋項目：

1. 中文 F 預覽只以單獨字元替換整個 preedit 顯示，沒有把既有組字與游標納入預覽；各鍵盤配置一律硬編碼 ㄑ。
2. ASCII 組字分支只 render 原有 buffer，尚未加入本次 f，無法滿足立即顯示的契約。
3. F timer 到期後直接取得當下 focused context；未像 initial hold 一樣驗證原始 context 與手勢目標。必須確認焦點、模式及取消時的狀態隔離。
4. timer 在 elapsed <= delay 時先移除自身後返回，沒有重新排程；可能不會在嚴格超過門檻後自動進層。
5. timer 先把狀態改 Held 後，下一個映射鍵略過原先 `!is_held` 的既有中文組字處理；須驗證不遺失舊組字，不可只以 enum 狀態測試當作輸入驗收。
6. Terminal 修正只以同 PID 作 HWND 根不同時的替代判斷；尚無 Zbook 實測證據，仍須區分焦點連線與宿主身分，不能直接宣稱修復。

這些為原始碼審查發現，尚非重現的 Windows 實機測試結果。不得將候選打包成已驗收版本。

## Zbook 委派

已通知既有「診斷並修復開始列搜尋輸入法」對話，核對已載入版本、Terminal 規則、前景 HWND/PID/exe 與 TSF 輸入視窗關係。只允許必要身分與旗標，不讀取終端內容、標題、命令歷史、私人詞庫或憑證；不安裝、重啟或改登錄。結果尚待回報。
