# Ubuntu AMD64 1.3.0 移植

目標：Ubuntu 24.04 LTS AMD64。開發分支 feature/ubuntu-amd64-1.3.0，從 Windows 1.3.0-dev.2 驗收來源 cab4b3c 分出；不混入詞彙卡 v2。

## 已確認

- 既有 Linux 補丁呼叫已不存在的 POLICY_PROFILE_MARKDOWN 及錯誤參數數目的 policy_transform_commit，不能直接編譯作為發行版。
- 原生 Windows TSF / 設定程式不能直接編譯為 Linux；Fcitx5 需要獨立前端、事件計時及組字整合。
- 上游 Fcitx5 Chewing 預設讀寫原版用戶詞庫；移植必須使用獨立 addon、設定及資料目錄，不能直接重新命名套件後出貨。
- Ubuntu 24.04 官方基準為 fcitx5-chewing 5.1.1、libchewing 0.6.0；現有 vendor manifest 的較新版本不能直接假定相容。
- WorkstationY 是 Ubuntu 26.04，使用者正在準備免密碼登入；預計以 Ubuntu 24.04 AMD64 容器建置，主機桌面測試另列為 26.04 相容性證據。

## 本輪準備

packaging/linux/ubuntu24 提供容器建置環境及既有政策核心／橋接基線測試腳本。僅供開發，不是安裝包。

## 尚待完成

- Ubuntu 容器實際建置與依賴驗證。
- 獨立 Fcitx5 addon、使用者詞庫隔離與安裝／移除。
- F keydown 原字預覽、長按撤回、Shift 與組字保留、焦點取消及 QWERTY 映射。
- APP 規則與 Wayland/X11 可識別範圍；不能套用 Windows exe 路徑。
- 設定介面、工程詞彙與 Ollama 功能逐項對齊及明確差異表。
- Ubuntu 24.04 實鍵驗收、AMD64 套件及授權清單；通過前使用測試後綴，不發布正式 1.3.0。

目前未安裝、未重啟輸入法、未推送或發布。Mac 無運行中的 Docker daemon，Ubuntu 編譯／桌面測試為 NOT_RUN。
