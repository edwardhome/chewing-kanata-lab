# Ubuntu 移植準備驗證

- Mac 本機 C 政策核心：32/32 斷言通過。
- Mac 本機 C++ Fcitx5 橋接：5/5 斷言通過。
- 編譯使用 -Wall -Wextra -Werror；C 原始碼獨立以 C99 編譯後連結 C++11。
- Ubuntu 腳本 bash -n：通過。
- Docker Ubuntu 24.04 AMD64 建置：NOT_RUN，Mac daemon 未運行。
- 原生 Fcitx5 模組、Linux F 長按及 APP 切換、桌面實鍵：NOT_RUN。

37 項為既有跨平台基線斷言，不是 Ubuntu 原生輸入法驗收，不計作新功能已完成。
