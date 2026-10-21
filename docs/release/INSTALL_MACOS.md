# macOS Fcitx5 Plugin 模組安裝手冊 (v1.1)

## 1. 路線說明
依據總規格書 P12 規範：macOS 端優先延續 Fcitx5-Chewing 外掛路線，共用跨平台 `policy-core`，不重新開發 InputMethodKit 前端。

## 2. 建置與打包
1. 需求工具：macOS 13.3+，Xcode 26.4+。
2. 執行建置腳本：
   ```bash
   bash packaging/macos/build_macos_addon.sh
   ```
3. 將產出之外掛套用至 `Fcitx5.app` 之插件目錄。

## 3. 卸載
- 於 Fcitx5 偏好設定中停用或自 Application Support 中移除外掛檔案。
