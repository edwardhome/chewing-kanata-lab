# Ollama 請求生命週期核心驗證

更新時間：2026-11-06T01:32:06+08:00

## 實作範圍

`policy-core/src/prediction_session.rs`：純邏輯排程、快照與世代比對、明確接受前重新核對上下文。不含網路、TSF、COM、GUI 或自動送字。

## 實際執行

- Mac `cargo test --manifest-path policy-core/Cargo.toml --locked`：BLOCKED_TOOLCHAIN，shell exit 127，找不到 cargo；未安裝或更改 Mac 輸入法。
- Windows rustfmt：BLOCKED_TOOLCHAIN，stable toolchain 沒有 rustfmt component。未將格式檢查記為 PASS。
- Windows `rustc 1.99.1 (58cf8f9c3 2026-10-13)`。
- 把本機模組複製到 Windows 暫存檔，用 `rustc --edition 2021 --test` 編譯，再執行測試 EXE：PASS，15 passed、0 failed，退出碼 0。
- 覆蓋停頓邊界、256 scalar 與非 BMP、停用／安全未知／未完成音節／選取與游標限制、最新等待項、取消後仍佔名額、重複文字不同世代、過期、單次明確接受、A/B 焦點／文件／模式、舊完成事件不可釋放新工作、時鐘倒退與世代溢位。

## 限制

單一在途請求限制是每個服務實例，不是跨應用全系統共用。工作者必須真正停止才回報 complete；此核心不負責強制停止網路工作。`valid_prediction` 必須來自 HTTP、JSON 及 AGY 前綴驗證全部通過的結果，不能直接相信模型。

Ollama 實際模型、HTTP transport、TSF 接線、設定頁、GUI 與實體按鍵：NOT_RUN。AI 開關與已安裝的 1.0.1 未變更。

## Git 整合與獨立複驗

- Codex 核心提交：`207ffc4`。
- AGY 原始 source 提交：`74dc146`；審查修正：`ec0d287`。兩者作者與提交者皆為 AGY。
- 審查要求 AGY 修正 ASCII-only 控制字元過濾，新增 C1 控制字元混合漢字案例，重寫真正的 32／33 字元邊界測試。修正後 prediction 模組 36 個測試；原有模組 37 個，AGY 單獨執行總計 73 個。
- 功能分支合併：`3bdbb2e`，保留 AGY 四個原始提交；跨模組整合：`1dcd371`。
- Windows 共用外層 `feature/ollama-phrase-prediction` 已 fast-forward 到 `1dcd371`；兩個 nested repo、main 與舊的未追蹤驗證報告均未修改。
- Codex 在整合後的 Windows 共用目錄獨立執行：

```powershell
$env:PATH += ';' + $env:USERPROFILE + '\.cargo\bin'
$env:CARGO_TARGET_DIR = Join-Path $env:TEMP 'ckai-codex-integrated-target'
cargo test --manifest-path policy-core/Cargo.toml --locked
```

實際結果：88 個單元測試通過（37 原有＋36 AGY＋15 Codex），5 個跨模組契約測試通過，合計 93 passed、0 failed；doc-tests 0 個，退出碼 0。完整輸出保存在 Windows `%TEMP%\ckai-ollama-integrated-tests.log`。

跨模組測試涵蓋原文直到明確接受前維持不變、改寫前綴拒絕、相同文字不同世代拒絕、接受前文件切換拒絕，以及 Unicode 前綴與首尾空白原樣保留。沒有以模型成功、GUI 成功或實鍵成功描述這些純邏輯測試。

## AGY 執行紀錄

本輪沿用同一互動式 SSH 工作階段。Gemini 3.8 Flash High 回傳 503 容量不足後，在同一對話使用 CLI 支援的單次模型選擇，改由可用的 Claude Sonnet 4.6 完成委派；未另開 AGY 程序。網頁只作過入口檢查，使用者要求後已停止 GUI 操作，後續委派、讀取回報及 Git 驗收皆走終端。
