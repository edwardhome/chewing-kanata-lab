# ADR-0003: 共用 C-ABI 政策庫 (`policy-core`) 決策

- **狀態**：已批准 (Approved)
- **日期**：2026-10-22

## 背景
Windows 前端為 Rust，Linux 前端為 C++，macOS 前端亦為 C/C++。各前端若各自寫一套符號轉換邏輯，容易導致規則發散與行為不一致。

## 決策
1. 實作純 C99 實作與 Rust 封裝之雙重介面。
2. 零外部依賴、純記憶體無鎖、時延小於 5ms。
3. 統一提供 `policy_should_bypass_key` 與 `policy_transform_commit`。
