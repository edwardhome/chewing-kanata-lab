# Linux Fcitx5 模組安裝與隔離部署手冊

## 1. 使用者個人目錄安裝 (推薦)
1. 進入工作區編譯 Fcitx5 Chewing 外掛：
   ```bash
   cd upstream/fcitx5-chewing
   git apply ../../adapters/fcitx5/adapter_fcitx5.patch
   cmake -S . -B build -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_INSTALL_PREFIX="$HOME/.local"
   cmake --build build --parallel
   ```
2. 執行隔離部署腳本：
   ```bash
   bash packaging/linux/install_user_addon.sh
   ```
3. 重啟 Fcitx5：
   ```bash
   fcitx5 -r &
   ```

## 2. 解除安裝
- 直接刪除 `$HOME/.local/lib/fcitx5/` 下對應的 chewing addon `.so`，系統全域安裝之 fcitx5 完全不受影響。
