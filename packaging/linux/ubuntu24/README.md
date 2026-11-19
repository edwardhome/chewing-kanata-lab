# Ubuntu 24.04 AMD64 預覽版

版本：`1.3.1-dev.ubuntu.1`。這是獨立 Fcitx5 原生注音前端的基礎預覽；桌面實鍵輸入尚待使用者驗收，不是 Windows 功能對等版。

## 下載及安裝

從 [GitHub 預覽版](https://github.com/edwardhome/chewing-kanata-lab/releases/tag/v1.3.1-dev.ubuntu.1) 下載 `AI-Chewing-Ubuntu24.04-amd64-1.3.1-dev.ubuntu.1.tar.gz` 與 `SHA256SUMS.txt`。在下載目錄先執行 `sha256sum -c SHA256SUMS.txt`，再解壓縮。

依賴由使用者自行安裝：

```sh
sudo apt install fcitx5 fcitx5-config-qt fcitx5-frontend-gtk3 fcitx5-frontend-gtk4 fcitx5-frontend-qt5 libchewing3 libchewing3-data python3
```

進入解壓縮的資料夾，以桌面使用者執行，**不要 sudo**：

```sh
python3 install-preview.py install
```

安裝器只註冊使用者目錄下的 `ai-chewing` 模組；不切換輸入法框架、不啟動或重啟 Fcitx5。請儲存工作後自行啟用 Fcitx5，再在 Fcitx5 設定加入 **AI Chewing (Ubuntu preview)**。若原本使用 IBus，切換框架的登入工作階段設定仍由使用者處理。

已安裝舊預覽版時，先回到舊包以其 `install-preview.py remove` 解除註冊，再安裝新版。移除同樣使用目前版本的原始包：

```sh
python3 install-preview.py remove
```

移除只刪除此版本管理的兩個註冊檔，保留詞庫、設定與版本程式庫。任何不屬此版本的註冊都會使操作停止，且不先刪除另一個檔案。

## 功能及資料範圍

- 已整合：原生注音、候選、上游設定介面，獨立模組及詞庫。無桌面驗收的部分不得視為已實鍵通過。
- 尚未移植：F 長按分層、APP 自動切換、工程詞彙管理、Ollama、GitHub 更新介面。
- 詞庫：`${XDG_DATA_HOME:-~/.local/share}/ai-chewing/userphrase.sqlite`。
- 設定：`${XDG_CONFIG_HOME:-~/.config}/fcitx5/conf/ai-chewing.conf`。
- 不讀寫原版新酷音個人詞庫。包內附上游授權及修改後完整對應原始碼。
- 此基礎包尚未攜帶上游翻譯與圖示資源；未安裝原版 fcitx5-chewing 時，設定可能顯示英文且缺少圖示。
- Ubuntu 26.04 主機僅用於承載建置；輸出在 Ubuntu 24.04 使用者空間編譯。26.04 桌面也尚未驗收。

## 自行建置

在 Ubuntu 24.04 AMD64 並已備齊編譯依賴的專案根目錄執行：

```sh
bash packaging/linux/ubuntu24/check_baseline.sh
bash packaging/linux/ubuntu24/build_addon.sh /path/to/empty/addon-build-1
cd /path/to/empty
cp /path/to/project/packaging/linux/ubuntu24/install-preview.py .
bash /path/to/project/packaging/linux/ubuntu24/package_preview.sh
```

`build_addon.sh` 固定取用 fcitx5-chewing `918156ad0157da66a8fd1afff5fbcad40e38a85b`（5.1.1），僅修改拋棄式副本，使用 libchewing 0.6.0 / Fcitx5 5.1.7 編譯。檔案存入 staging，不安裝至系統。

`Dockerfile` 提供另一個基礎環境，預設只跑政策核心與橋接測試。這些橋接尚未接入原生輸入法，不能代表 F 層或 APP 規則已完成。

驗證記錄見 [本次發行驗證](../../../docs/verification/runs/RUN-20261119-UBUNTU-PREVIEW-131.md)。
