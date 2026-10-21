# 建置、執行與隔離安裝手冊 (Build and Run Guide)

> **目標**：提供清晰、可重複驗證的建置與隔離執行步驟。  
> **安全宣告**：本專案嚴格遵循「不得覆蓋系統正式新酷音與個人詞庫」之要求。

---

## 1. 策略核心與單元測試建置 (當前主機即刻可用)

### 1.1 需求工具
- GCC / MinGW (`gcc.exe`, `g++.exe` >= 6.3) 或 MSVC (`cl.exe`)。

### 1.2 編譯與執行 `policy-core` 單元測試
```powershell
cd chewing-kanata-lab

# 編譯 C-ABI 核心與單元測試
gcc -std=c99 -Wall -Wextra -Ipolicy-core\include policy-core\src\policy_core.c policy-core\tests\test_policy.c -o policy-core\tests\test_policy.exe

# 執行測試 (32 項斷言)
.\policy-core\tests\test_policy.exe
```

### 1.3 編譯與執行 `chewing-adapter-fcitx5` 單元測試
```powershell
# 編譯 C++11 Adapter 與單元測試
g++ -std=c++11 -Wall -Wextra -Ipolicy-core\include -Iadapters\fcitx5\include policy-core\src\policy_core.c adapters\fcitx5\src\fcitx5_policy_bridge.cpp adapters\fcitx5\tests\test_fcitx5_adapter.cpp -o adapters\fcitx5\tests\test_fcitx5_adapter.exe

# 執行測試 (5 項斷言)
.\adapters\fcitx5\tests\test_fcitx5_adapter.exe
```

---

## 2. Windows TSF 模組建置與隔離安裝

### 2.1 需求工具
- Rust 工具鏈 (Cargo, Edition 2024 / MSRV 1.95+)
- Visual Studio 2022 / 2026 Build Tools (C++ 與 Windows SDK)

### 2.2 建置命令 (於乾淨開發環境)
```powershell
cd chewing-kanata-lab\upstream\windows-chewing-tsf

# 套用本專案之 Adapter 最小補丁
git apply ..\..\adapters\windows-tsf\adapter_tsf.patch

# 透過 xtask 建置獨立開發版安裝器 (獨立 GUID，不覆蓋正式版)
cargo xtask build-installer --target msvc --release
```

### 2.3 隔離執行與一鍵卸載
- **設定檔位置**：`%APPDATA%\ChewingDev\policy.toml`
- **一鍵回退**：將 `policy.toml` 中之 `default_profile` 設為 `"legacy"`，即可 100% 恢復原生新酷音行為。
- **卸載**：直接於控制台解除安裝 `Chewing ASCII-First Dev (TSF)`，系統原有新酷音完全不受影響。

---

## 3. Linux Fcitx5 模組建置 (Linux 開發環境)

### 3.1 需求套件
- CMake >= 3.6, ECM (Extra CMake Modules)
- `fcitx5-dev`, `libchewing-dev`

### 3.2 建置步驟
```bash
cd chewing-kanata-lab/upstream/fcitx5-chewing

# 套用 Adapter 補丁
git apply ../../adapters/fcitx5/adapter_fcitx5.patch

# 建置並安裝至使用者個人目錄 (~/.local)
cmake -S . -B build -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_INSTALL_PREFIX="$HOME/.local"
cmake --build build --parallel

# 複製 policy 設定
mkdir -p ~/.config/fcitx5
cp ../../config/policy.example.toml ~/.config/fcitx5/chewing-policy.toml
```
