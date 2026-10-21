# ADR-0002: 優先採用 Adapter-first 策略架構決策

- **狀態**：已批准 (Approved)
- **日期**：2026-10-22
- **決策者**：Antigravity Engineering Agent

## 背景
`libchewing` 0.14.x 正在進行大規模 Rust 重構。若直接 Fork 修改其內核 `special_symbol_input`，未來每一次上游更新拼音/詞庫時將承受巨大的 Merge 衝突。

## 決策
1. 嚴格採行 **Adapter-first** 原則。
2. 攔截點設置於前端：Windows TSF 的 `on_test_keydown` 放行閘門與 Linux Fcitx5 的 `ChewingEngine::keyEvent`。
3. 組字區為空時直接 Bypass 放行語法鍵；組字中若產生符號則於 Commit 前過濾。
4. 核心 `libchewing` 維持 100% 原始上游代碼，零相依包袱。
