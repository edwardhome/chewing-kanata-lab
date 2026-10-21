#!/usr/bin/env bash
# Linux Fcitx5 Chewing User-Local Install Script
# 安裝至使用者個人目錄 (~/.local)，嚴禁污染系統全域目錄 (/usr)

set -euo pipefail

PREFIX="${HOME}/.local"
BUILD_DIR="upstream/fcitx5-chewing/build"

echo "=== 安裝 Fcitx5 Chewing (ASCII-First Dev) 至 ${PREFIX} ==="

if [[ ! -d "${BUILD_DIR}" ]]; then
    echo "錯誤: 建置目錄不存在，請先執行 cmake --build"
    exit 1
fi

cmake --install "${BUILD_DIR}" --prefix "${PREFIX}"

mkdir -p "${HOME}/.config/fcitx5"
if [[ ! -f "${HOME}/.config/fcitx5/chewing-policy.toml" ]]; then
    cp config/policy.example.toml "${HOME}/.config/fcitx5/chewing-policy.toml"
    echo "已部署範例設定檔至 ~/.config/fcitx5/chewing-policy.toml"
fi

echo "安裝完成。請重啟 fcitx5: 'fcitx5 -r &'"
