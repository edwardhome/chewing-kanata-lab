# Ubuntu 24.04 AMD64 開發環境

目前僅為 Linux 移植建置環境，不是可安裝的 1.3.0 發行版。主機可為 Ubuntu 26.04，但輸出相容性以 Ubuntu 24.04 容器為準。

在專案根目錄：

```sh
docker build --platform linux/amd64 -f packaging/linux/ubuntu24/Dockerfile -t ai-chewing-ubuntu24-build .
docker run --rm --platform linux/amd64 --network none ai-chewing-ubuntu24-build
```

此流程僅編譯並執行既有 C 政策核心與 C++ 橋接測試，不安裝、不啟動 Fcitx5、不改主機輸入法及使用者詞庫。不能將基礎測試通過視為 F 分層、APP 切換、Ollama 或完整輸入法已可用。

Ubuntu 官方套件基準：fcitx5-chewing 5.1.1-1、libchewing 0.6.0-1build1。

- https://packages.ubuntu.com/noble/fcitx5-chewing
- https://packages.ubuntu.com/source/noble/libchewing

已唯讀取得的上游 5.1.1 commit：918156ad0157da66a8fd1afff5fbcad40e38a85b。既有 vendor-manifest.lock 的 5.1.13 不擅自改寫，後續 Linux 發行需獨立鎖定來源。
