# Ubuntu 1.3.1-dev.ubuntu.1 發行驗證

更新時間：2026-11-19T11:12:37+08:00

使用者已驗收 Windows 1.3.1，並授權 Windows 正式版與 Linux 發布。Linux 尚無桌面驗收，依既有命名規則公開為預覽版；不能以 Windows 的驗收替代 Linux 測試。

## 來源與範圍

來源分支 `feature/ubuntu-amd64-1.3.0` 已合入 Windows 正式來源 `c168e69`，未混入詞彙卡 v2。原生 Linux 使用固定上游 fcitx5-chewing `918156ad0157da66a8fd1afff5fbcad40e38a85b`，經 `prepare_source.py` 產生獨立 addon、設定及詞庫；修改後完整對應來源與 LGPL-2.1-or-later 隨包提供。

本輪強化使用者安裝器：強制六項完整且唯一的校驗清單，拒絕目標符號連結／斷鏈，移除前驗證全部註冊，使用排他建立避免覆寫後來出現的檔案。保留使用者詞庫、設定及舊版程式庫。不自動啟動或重啟 Fcitx5。

## 實際測試

WorkstationY 是 Ubuntu 26.04.1 x86_64；原生模組在獨立 Ubuntu 24.04.5 AMD64 PRoot 使用者空間重新編譯，未安裝至主機。GCC 13、Fcitx5 5.1.7、libchewing 0.6.0。

- C 政策核心：32 項斷言通過。
- C++ 橋接：5 組案例通過。此橋接尚未接入原生前端，不能視為 F／APP 功能已實作。
- 安裝器：15 個 unittest 通過；在 Ubuntu 26.04 主機及 Ubuntu 24.04 PRoot 非 root 身分各跑一次。均使用臨時 HOME、XDG 和複製的測試包，每次操作檢查合成詞庫未改變。兩輪是相同 15 案例，不重複計數。
- 原生 Release 編譯、staging、ELF64 AMD64 與 `ldd` 依賴解析：通過。
- 直接連結實際原生物件的合成環境測試：模組載入、原生 F 注音產生、獨立 ai-chewing 詞庫建立通過，原版 chewing 資料目錄未建立。
- 六項包內 SHA256SUMS、Mac 下載後整包 SHA-256 與壓縮檔目錄檢查通過。
- 桌面輸入、實體按鍵、焦點／候選視窗與 Wayland／X11：NOT_RUN，由使用者安裝測試。

## 已知缺項

F 長按分層、APP 自動切換、工程詞彙管理、Ollama 與更新介面尚未移植；本包不代表 Windows 1.3.1 功能對等。尚未攜帶上游翻譯與圖示資源；未裝原版 fcitx5-chewing 時設定可能為英文且缺少圖示。

## 成品

- 檔名：`AI-Chewing-Ubuntu24.04-amd64-1.3.1-dev.ubuntu.1.tar.gz`
- 大小：82231 bytes
- SHA-256：`4b5243217105dfb21895823db6221135f34687907c6336bbcb437f43f62cd5d6`
- 本機：`/Users/defu/Documents/AI-Chewing/Ubuntu/1.3.1-dev.ubuntu.1/`
- WorkstationY：`~/ckai-ubuntu24-build/work/release-131/`

Windows 正式版 `v1.3.1` 已公開，保留為 Latest；Linux 使用獨立 tag `v1.3.1-dev.ubuntu.1` 與 prerelease，不替換 Windows 更新來源。

## 本次日誌

- [addon-build.log](ubuntu-1.3.1-dev.ubuntu.1/addon-build.log) SHA-256 `74ea3c542b630ac674cd5b94d09017615bb834637bc6b479997e10b269894b6e`
- [baseline.log](ubuntu-1.3.1-dev.ubuntu.1/baseline.log) SHA-256 `af9f86bdd004093948a24b1c94b2883e3ad9d9b78d6cdc90097e52a313c3d039`
- [installer-noble-test.log](ubuntu-1.3.1-dev.ubuntu.1/installer-noble-test.log) SHA-256 `1a4b742cbef7a6853e7ac473a4340a8febb5cb04fa2eb62c8733426d5c0389e7`
- [installer-test.log](ubuntu-1.3.1-dev.ubuntu.1/installer-test.log) SHA-256 `84652fa159b0fe7a95a6e2c9e97233c1bb21902d1f3b1178823c32bb30199173`
- [package.log](ubuntu-1.3.1-dev.ubuntu.1/package.log) SHA-256 `86e49a4b8d4c792a7c9da0463589411aea91a6a569c8fed438b7c5d826abb325`
- [smoke.log](ubuntu-1.3.1-dev.ubuntu.1/smoke.log) SHA-256 `14ea9410b8f7256e8c3e1af74160b5dca746eb9a8f0a0a2ba31b09b60dab2db7`
