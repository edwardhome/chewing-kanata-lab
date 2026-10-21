# 跨平台符號策略核心規格書 (Policy Core Architecture & Algorithm)

> **模組名稱**：`policy-core` (`chewing-policy-core`)  
> **程式語言**：Rust (Edition 2021) + 零依賴 C-ABI  
> **目標**：提供一個可同時被 Windows TSF (Rust) 與 Linux Fcitx5 (C++) 共同調用的純同步、零系統依賴、高確定性符號決策庫。

---

## 1. 模組定位與架構設計

```mermaid
flowchart LR
    subgraph Core["policy-core (純邏輯層)"]
        Table["符號分類查表 (Syntax, Punctuation, Fullwidth)"]
        FSM["Profile 狀態機 (Profile State Machine)"]
        Matcher["上下文與安全欄位過濾器"]
    end

    subgraph Consumer1["Windows TSF (Rust)"]
        W_CTS["ChewingTextService"]
    end

    subgraph Consumer2["Linux Fcitx5 (C++)"]
        L_EIM["ChewingEngine (C-ABI)"]
    end

    W_CTS -->|Rust crate 直接呼叫| Core
    L_EIM -->|include/policy_core.h (動態/靜態庫)| Core
```

### 設計約束：
1. **零外部網路呼叫**：不包含任何 HTTP/Socket/Ollama 調用，保證執行時延在微秒（$\mu s$）等級。
2. **零系統 API 依賴**：不引用 `windows-sys` 或 `fcitx`，保證跨平台 100% 可編譯與純演算法確定性。
3. **無鎖與同步**：所有決策函式均為同步無鎖函式，不阻塞 IME 前端事件主迴圈。

---

## 2. 符號分類與查表規格 (Symbol Tables)

### 2.1 語法關鍵字元集 (Syntax Punctuation Set)
在 `markdown` 與 `code` 模式下，以下字元在無組字緩衝區時**一律放行 (Bypass)**：
```text
括號類：  (  )  [  ]  {  }  <  >
語法類：  :  ;  `  _  |  =  +  -  *  /  \  ~  ^  %  $  #  @  !  ?  "  '
```

### 2.2 繁體中文正式書信標點對照表 (Traditional Chinese Formal Map)
在 `formal` Profile 或使用者明確要求中文全形標點時，支援以下標準台灣繁體中文標點：

| ASCII 輸入 | 正式中文標點 | Unicode 碼點 | 備註說明 |
| :---: | :---: | :---: | :--- |
| `:` | `：` | `U+FF1A` | 全形冒號 |
| `;` | `；` | `U+FF1B` | 全形分號 |
| `,` | `，` | `U+FF0C` | 全形逗號 |
| `.` | `。` | `U+3002` | 全形句號 |
| `?` | `？` | `U+FF1F` | 全形問號 |
| `!` | `！` | `U+FF01` | 全形驚嘆號 |
| `[` | `「` | `U+300C` | 繁體中文引號（開） |
| `]` | `」` | `U+300D` | 繁體中文引號（關） |
| `{` | `『` | `U+300E` | 繁體中文雙引號（開） |
| `}` | `』` | `U+300F` | 繁體中文雙引號（關） |
| `(` | `（` | `U+FF08` | 全形圓括號（開） |
| `)` | `）` | `U+FF09` | 全形圓括號（關） |
| `_` | `—` | `U+2014` | 破折號 |
| `~` | `～` | `U+FF5E` | 全形波浪號 |
| `\` | `、` | `U+3001` | 頓號 |

---

## 3. 核心決策演算法 (Decision Algorithms)

### 3.1 按鍵放行判定：`should_bypass_key`
```text
演算法：should_bypass_key(ascii_key, context)
輸入：ascii_key (char), context (PolicyContext)
輸出：boolean (true 代表放行給應用程式，false 代表由 IME 吸收處理)

1. 若 context.is_secure_field == true：
     返回 true (安全欄位絕不介入)
2. 若 context.profile == RAW_COMMAND：
     返回 true (命令模式完全放行)
3. 若 context.profile == MANUAL_FULLWIDTH：
     返回 false (全形模式交由 IME 處理)
4. 若 context.has_active_preedit == true：
     返回 false (正在組字中，按鍵需交由 IME 進行組字/選字)
5. 根據 context.profile 判定：
     a. 若 profile == CODE 或 MARKDOWN：
          若 ascii_key 屬於 Syntax Punctuation Set：
              返回 true
     b. 若 profile == DEFAULT：
          若 ascii_key 屬於高頻程式括號冒號 (':', '[', ']', '{', '}', '`', '_')：
              返回 true
     c. 若 profile == FORMAL：
          返回 false (正式模式由 IME 提供中文全形標點)
6. 預設返回 false
```

### 3.2 提交轉換判定：`transform_commit`
```text
演算法：transform_commit(input_str, context)
輸入：input_str (&str), context (PolicyContext)
輸出：&str (轉換後字串)

1. 若 context.is_secure_field == true 或 context.profile == RAW_COMMAND：
     返回 input_str
2. 若 context.profile == MARKDOWN 或 CODE：
     若 input_str 等於 "："：返回 ":"
     若 input_str 等於 "「"：返回 "["
     若 input_str 等於 "」"：返回 "]"
     若 input_str 等於 "（"：返回 "("
     若 input_str 等於 "）"：返回 ")"
3. 返回 input_str
```

---

## 4. C-ABI 介面規格 (`policy_core.h`)

```c
#ifndef CHEWING_POLICY_CORE_H
#define CHEWING_POLICY_CORE_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    POLICY_PROFILE_DEFAULT = 0,
    POLICY_PROFILE_MARKDOWN = 1,
    POLICY_PROFILE_CODE = 2,
    POLICY_PROFILE_FORMAL = 3,
    POLICY_PROFILE_RAW_COMMAND = 4,
    POLICY_PROFILE_MANUAL_FULLWIDTH = 5
} policy_profile_t;

typedef struct {
    policy_profile_t profile;
    bool has_active_preedit;
    bool is_secure_field;
} policy_context_t;

/* 判斷特定 ASCII 鍵在目前 Profile 與上下文下是否應直接放行給應用程式 */
bool policy_should_bypass_key(char ascii_key, const policy_context_t *ctx);

/* 針對 Commit 字串進行策略轉換 */
const char* policy_transform_commit(const char *utf8_text, const policy_context_t *ctx);

#ifdef __cplusplus
}
#endif

#endif /* CHEWING_POLICY_CORE_H */
```
