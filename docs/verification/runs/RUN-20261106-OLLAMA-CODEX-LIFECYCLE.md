# Ollama 請求生命週期核心驗證

更新時間：2026-11-06T01:16:43+08:00

## 實作範圍

`policy-core/src/prediction_session.rs`：純邏輯排程、快照與世代比對、明確接受前重新核對上下文。不含網路、TSF、COM、GUI 或自動送字。

## 實際執行

- Mac `cargo test --manifest-path policy-core/Cargo.toml --locked`：BLOCKED_TOOLCHAIN，shell exit 127，找不到 cargo；未安裝或更改 Mac 輸入法。
- Windows rustfmt：BLOCKED_TOOLCHAIN，stable toolchain 沒有 rustfmt component。未將格式檢查記為 PASS。
- Windows `rustc 1.99.1 (58cf8f9c3 2026-10-13)`。
- 把本機模組複製到 Windows 暫存檔，用 `rustc --edition 2021 --test` 編譯，再執行測試 EXE：PASS，15 passed、0 failed，退出碼 0。
- 覆蓋停頓邊界、256 scalar 與非 BMP、停用／安全未知／未完成音節／選取與游標限制、最新等待項、取消後仍佔名額、重複文字不同世代、過期、單次明確接受、A/B 焦點／文件／模式、舊完成事件不可釋放新工作、時鐘倒退與世代溢位。

## 限制

整體 policy-core Cargo 測試待 AGY 模組整合後執行。單一在途請求限制是每個服務實例，不是跨應用全系統共用。工作者必須真正停止才回報 complete；此核心不負責強制停止網路工作。`valid_prediction` 必須來自 HTTP、JSON 及 AGY 前綴驗證全部通過的結果，不能直接相信模型。

Ollama 實際模型、HTTP transport、TSF 接線、設定頁、GUI 與實體按鍵：NOT_RUN。AI 開關與已安裝的 1.0.1 未變更。
