#!/usr/bin/env bash
set -euo pipefail
# Run inside the disposable Ubuntu build workspace, after build_addon.sh.
# addon-build-1 and install-preview.py must be present in the current directory.
p=AI-Chewing-Ubuntu24.04-amd64-1.3.1-dev.ubuntu.1
mkdir "$p"
cp addon-build-1/stage/usr/lib/fcitx5/libai-chewing.so "$p/"
cp addon-build-1/stage/usr/share/fcitx5/inputmethod/ai-chewing.conf "$p/"
cp install-preview.py "$p/"
cp addon-build-1/source/LICENSES/LGPL-2.1-or-later.txt "$p/UPSTREAM-COPYING"
tar --exclude=.git -czf "$p/corresponding-source.tar.gz" -C addon-build-1 source
cat > "$p/README.txt" <<'EOF'
AI新酷音 Ubuntu 24.04 AMD64 原生前端基礎測試包
版本：1.3.1-dev.ubuntu.1

這不是完整 Windows 1.3.1 功能對等版。
已提供：獨立 Fcitx5 新酷音模組、原生注音/候選/設定、獨立使用者詞庫。
尚未提供：F 長按分層、APP 自動切換、工程詞彙管理、Ollama 預測、GitHub 更新介面。

1. Ubuntu 24.04 依賴（由使用者操作）：
   sudo apt install fcitx5 fcitx5-config-qt fcitx5-frontend-gtk3 fcitx5-frontend-gtk4 fcitx5-frontend-qt5 libchewing3 libchewing3-data python3
2. 解壓縮後，以桌面使用者執行，不要 sudo：
   python3 install-preview.py install
   已裝舊預覽版者，先用舊包的 install-preview.py remove 解除註冊，再安裝新版；詞庫保留。
3. 請儲存工作，自行啟用/重新啟動 Fcitx5，再於 Fcitx5 設定加入 AI Chewing (Ubuntu preview)。
   若原本使用 IBus，切換框架涉及登入工作階段設定；此安裝器不代為修改。
4. 測試記事本/瀏覽器/終端機的注音、空白選字、Enter、退格、切換視窗。
5. 移除本預覽註冊：python3 install-preview.py remove
   不刪除詞庫或設定，待使用者自行重啟 Fcitx5。

詞庫：${XDG_DATA_HOME:-~/.local/share}/ai-chewing/userphrase.sqlite
設定：${XDG_CONFIG_HOME:-~/.config}/fcitx5/conf/ai-chewing.conf
不修改原版新酷音詞庫，不自動啟動或重啟輸入法。

編譯：Ubuntu 24.04 AMD64 使用者空間、GCC 13、Fcitx5 5.1.7、libchewing 0.6.0。
桌面實機測試：尚未執行。Ubuntu 26.04 桌面亦未驗收。
上游：fcitx/fcitx5-chewing commit 918156ad0157da66a8fd1afff5fbcad40e38a85b
授權：LGPL-2.1-or-later，原始著作權及對應修改來源隨附。
EOF
(cd "$p" && sha256sum libai-chewing.so ai-chewing.conf install-preview.py UPSTREAM-COPYING corresponding-source.tar.gz README.txt > SHA256SUMS)
tar -czf "$p.tar.gz" "$p"
sha256sum "$p.tar.gz"
