# ADR-0001: 原始碼基線、儲存庫與授權隔離決策

- **狀態**：已批准 (Approved)
- **日期**：2026-10-22
- **決策者**：Antigravity Engineering Agent

## 背景
專案整合了多個不同開源專案，授權涵蓋 LGPL-2.1-or-later (`libchewing`, `fcitx5-chewing`)、GPL-3.0-or-later (`windows-chewing-tsf`) 與 LGPL-3.0-only (`kanata`)。

## 決策
1. 以 Codeberg 為主要上游，GitHub 鏡像為備援，所有依賴鎖定於 `vendor-manifest.lock`。
2. 專案新增的純邏輯模組 (`policy-core`) 採寬鬆授權 (MIT OR Apache-2.0)，可同時被 GPL-3.0 與 LGPL-2.1 前端引用而無授權污染。
3. 嚴禁合併為單一不明授權，嚴禁覆寫上游 LICENSE / COPYRIGHT 聲明。
