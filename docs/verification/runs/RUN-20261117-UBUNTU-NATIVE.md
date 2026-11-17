# Ubuntu 24.04 AMD64 原生前端編譯

更新時間：2026-11-17T14:46:55+08:00

## 結論與限制

原生前端基礎測試包 1.3.0-dev.ubuntu.1 已編譯。不是 Windows 1.3.0 功能對等版；F 長按、APP 規則、工程詞彙管理、Ollama 與更新介面尚未移植。桌面輸入驗收 NOT_RUN，不發布正式版。

## 實際環境

WorkstationY：Ubuntu 26.04 x86_64。建置在使用者目錄 ckai-ubuntu24-build/rootfs 的 Ubuntu Base 24.04.5 AMD64；下載自 Canonical ubuntu-base release，SHA256SUMS 驗證成功。PRoot 5.4.0 執行使用者空間，沒有取得管理權限或改變主機系統套件。PRoot 預設加速模式會 SIGSEGV，使用 PROOT_NO_SECCOMP=1 關閉其追蹤加速後可運行；沒有修改主機核心安全設定。

GCC 13、CMake 3.28.3、Fcitx5 5.1.7、libchewing 0.6.0、glibc 2.39 均由 Ubuntu 24.04 套件來源安裝至該 rootfs。不是在 Ubuntu 26.04 原生編譯後宣稱相容 24.04。

## 結果

- C 政策核心：32/32 斷言通過。
- C++ 橋接：5 組案例通過（內含多個斷言）。此橋接尚未接入新版原生前端，不表示政策功能已在桌面啟用。
- 原生 Release 編譯及 staging：PASS；檔案 ELF64 x86-64，ldd 所有依賴均解析成功。
- 合成 HOME/XDG_DATA_HOME smoke：模組 dlopen 成功；連結實際原生物件呼叫其私有建構函式，F 產生注音，建立 ai-chewing/userphrase.sqlite，沒有建立原版資料目錄。私有函式沒有對外輸出，測試改為直接連結實際物件，不為測試擴大產品匯出介面。
- 安裝器臨時目錄測試：安裝、絕對載入路徑、重複註冊拒絕、移除保留詞庫、保護非本工具註冊共5項情境 PASS。
- 沒有啟動 Fcitx5、沒有安裝至使用者真實設定、沒有改動原版詞庫。

## 來源及交付

上游 fcitx5-chewing：918156ad0157da66a8fd1afff5fbcad40e38a85b。prepare_source.py 要求來源完全乾淨且固定 commit，僅在拋棄式 checkout 套用獨立模組/資料目錄修正。套件附修改後完整對應原始碼及 LGPL-2.1-or-later。

AI-Chewing-Ubuntu24.04-amd64-1.3.0-dev.ubuntu.1.tar.gz
SHA-256：cc5a9182fef8d709784543bf445990d6f94c21231948afa2914ba4a5a5c3e7f8

Mac 交付位置：/Users/defu/Documents/AI-Chewing/Ubuntu/1.3.0-dev.ubuntu.1/
WorkstationY：~/ckai-ubuntu24-build/work/ 同名檔案。

建置日誌：WorkstationY ~/ckai-ubuntu24-build/work/baseline.log、addon-build.log；摘要日誌 SHA-256：
- baseline.log：af9f86bdd004093948a24b1c94b2883e3ad9d9b78d6cdc90097e52a313c3d039
- addon-build.log：12b9d93d3ee3a2102dfe47e8b4cd9b3f58029a6a5559cd9adef26110ae758d52
