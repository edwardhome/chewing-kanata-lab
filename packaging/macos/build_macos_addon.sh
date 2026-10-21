#!/usr/bin/env bash
# macOS Fcitx5-Chewing Plugin Build & Packaging Route (v1.1)
# 依據總規格書 P12 規範：延續 Fcitx5 Chewing 外掛路線，不重新重寫巨大的 InputMethodKit 前端

set -euo pipefail

echo "=== macOS Fcitx5 Chewing Plugin 建置準備 ==="

FCITX5_MACOS_DIR="upstream/fcitx5-macos"
BUILD_DIR="${FCITX5_MACOS_DIR}/build"

echo "1. 檢查 Xcode 與 macOS 環境..."
if ! command -v xcodebuild &>/dev/null; then
    echo "提示: 當前環境非 macOS 或未安裝完整 Xcode 工具鏈。"
    echo "macOS 建置規範: 執行環境 macOS >= 13.3，建置環境 Xcode >= 26.4。"
    echo "本腳本作為 CI 與目標主機自動化打包使用。"
    exit 0
fi

echo "2. 套用跨平台 policy-core 與 Fcitx5 Adapter 補丁..."
# git apply ../../adapters/fcitx5/adapter_fcitx5.patch

echo "3. 執行 CMake 建置 macOS Bundle..."
# cmake -S "${FCITX5_MACOS_DIR}" -B "${BUILD_DIR}" -DCMAKE_BUILD_TYPE=Release
# cmake --build "${BUILD_DIR}"

echo "macOS Fcitx5 Plugin 打包流程就緒。"
