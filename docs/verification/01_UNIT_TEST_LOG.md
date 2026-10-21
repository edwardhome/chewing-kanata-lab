# 單元測試執行紀錄 (Unit Test Log)

> **執行日期**：2026-10-22  
> **作業系統**：Windows 10/11 (x86_64)  
> **編譯工具**：MinGW GCC 6.3.0 (`C:\MinGW\bin\gcc.exe`, `C:\MinGW\bin\g++.exe`)  
> **測試目標**：
> 1. `policy-core` C99 實作與 C-ABI (32 項斷言)
> 2. `chewing-adapter-fcitx5` C++11 實作 (5 項斷言)

---

## 1. 測試一：`policy-core` 純邏輯核心

### 1.1 編譯指令
```powershell
C:\MinGW\bin\gcc.exe -std=c99 -Wall -Wextra -Ipolicy-core\include policy-core\src\policy_core.c policy-core\tests\test_policy.c -o policy-core\tests\test_policy.exe
```

### 1.2 執行與終端輸出
```text
=========================================
 Chewing Policy Core Unit Test Suite
=========================================
Running test_default_context...
  [PASS] Default mode must be LEGACY for safe fallback
  [PASS] Default preedit must be false
  [PASS] Default is_secure_field must be false
  [PASS] Default editor mode must be UNKNOWN
Running test_ascii_first_bypass...
  [PASS] Colon ':' should bypass in ASCII-first
  [PASS] Left bracket '[' should bypass in ASCII-first
  [PASS] Right bracket ']' should bypass in ASCII-first
  [PASS] Left brace '{' should bypass in ASCII-first
  [PASS] Right brace '}' should bypass in ASCII-first
  [PASS] Left paren '(' should bypass in ASCII-first
  [PASS] Right paren ')' should bypass in ASCII-first
  [PASS] Backtick '`' should bypass in ASCII-first
  [PASS] Underscore '_' should bypass in ASCII-first
  [PASS] Semicolon ';' should bypass in ASCII-first
  [PASS] Colon ':' must NOT bypass when preedit is active
  [PASS] Left bracket '[' must NOT bypass when preedit is active
Running test_legacy_mode_fallback...
  [PASS] Colon must NOT bypass in legacy mode
  [PASS] Bracket must NOT bypass in legacy mode
  [PASS] Legacy mode must keep fullwidth colon untouched
Running test_formal_zh_mode...
  [PASS] Formal mode should not bypass key
  [PASS] Formal mode must preserve fullwidth colon
  [PASS] Formal mode must preserve opening quote
Running test_secure_and_vim_modes...
  [PASS] Secure field must bypass all keys
  [PASS] Vim Normal mode must bypass 'd'
  [PASS] Vim Normal mode must bypass ':'
  [PASS] Vim Command mode must bypass 'w'
Running test_transform_commit...
  [PASS] Transform '：' to ':' in ascii_first mode
  [PASS] Transform '「' to '[' in ascii_first mode
  [PASS] Transform '」' to ']' in ascii_first mode
  [PASS] Chinese phrases must remain 100% untouched
Running test_reranker_stub...
  [PASS] Reranker stub should return success (0)
  [PASS] Reranker stub must preserve candidate order
=========================================
 Test Results: 32 / 32 passed.
=========================================
```
* **結果**：32 / 32 測試通過，退出碼 0。

---

## 2. 測試二：`chewing-adapter-fcitx5` C++ 轉接層

### 2.1 編譯指令
```powershell
C:\MinGW\bin\g++.exe -std=c++11 -Wall -Wextra -Ipolicy-core\include -Iadapters\fcitx5\include policy-core\src\policy_core.c adapters\fcitx5\src\fcitx5_policy_bridge.cpp adapters\fcitx5\tests\test_fcitx5_adapter.cpp -o adapters\fcitx5\tests\test_fcitx5_adapter.exe
```

### 2.2 執行與終端輸出
```text
=========================================
 Fcitx5 Adapter Bridge Unit Test Suite
=========================================
  [PASS] ASCII syntax keys bypass when preedit is empty
  [PASS] Keys are not bypassed when preedit is active
  [PASS] Transform fullwidth colon to ascii in ASCII_FIRST
  [PASS] Legacy mode preserves original fullwidth colon
  [PASS] Chinese phrase remains 100% untouched
=========================================
 All Fcitx5 Adapter Tests Passed!
=========================================
```
* **結果**：5 / 5 測試通過，退出碼 0。
