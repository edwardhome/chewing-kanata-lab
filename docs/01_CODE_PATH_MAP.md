# 跨平台輸入法程式碼路徑地圖 (Code Path Map)

> **文件狀態**：原始碼靜態追蹤地圖  
> **基準版本**：
> - `libchewing`: `0.14.0-alpha.4` (`ec7f595ea7726fa06a9493e819b81669eba605a5`)
> - `windows-chewing-tsf`: `26.9.0` (`053d31ba10c089ae98508dd4a908ac2c2be3cb49`)
> - `fcitx5-chewing`: `5.1.13` (`60096e8ef49df21de6c155d7a0785a58fcccb955`)
> - `ibus-chewing`: `2.2.0` (`6e885f1a8556cb8cfcc657c5b0cfd71b93db4c5c`)
> - `kanata`: `kanata.original.kbd` (使用者原始配置)

---

## 1. 概念流向與邊界控制層級

```mermaid
flowchart TD
    subgraph Layer0["層級 0: 硬體與改鍵層 (Kanata)"]
        PK["實體按鍵 (Physical Key)"] --> KT["Kanata Engine (kanata.original.kbd)"]
        KT -->|標準按鍵 / 虛擬鍵碼| OSK["OS 鍵盤事件佇列 (Win32 / evdev / X11)"]
        KT -->|Unicode 注入動作 (u+003a)| OSU["OS 直接注入文字路徑 (SendInput UNICODE / Ctrl+Shift+U)"]
    end

    subgraph Layer1["層級 1: 平台輸入法前端 (IME Adapter)"]
        OSK -->|KeyEvent / TSF callback| FE["前端適配器 (Windows TSF / Fcitx5 / IBus)"]
        FE -->|預檢過濾 (Shortcut / Passthrough)| APP0["直接放行給應用程式"]
        FE -->|可組字事件| LC["核心組字引擎 (libchewing)"]
    end

    subgraph Layer2["層級 2: 核心組字引擎 (libchewing)"]
        LC --> SM{"特殊符號查表 (special_symbol_input)"}
        SM -->|命中符號 (如 ':' -> '：')| BUF["注入組字緩衝區 (shared.com.insert)"]
        SM -->|未命中 / 字母數字| CF{"字形模式 (CharacterForm)"}
        CF -->|Halfwidth| ASCII["半形 ASCII / 注音拼音"]
        CF -->|Fullwidth| FULL["全形轉換 (full_width_symbol_input)"]
    end

    subgraph Layer3["層級 3: 輸出與提交 (Preedit / Commit)"]
        BUF --> COM["產生 Commit 字串 / Preedit 更新"]
        ASCII --> COM
        FULL --> COM
        COM --> APPOUT["平台 API 提交至應用程式 (TSF InsertText / Fcitx5 commitString)"]
    end

    subgraph Layer4["層級 4: 應用程式自身後處理 (Out-of-Scope)"]
        OSU -.->|直接送達| APP["目標應用程式 (VS Code / Neovim / Obsidian / Outlook)"]
        APPOUT -.->|提交文字| APP
        APP --> AR{"應用程式自動替換 (Auto-correct / Smart Punctuation)"}
        AR --> RENDER["最終顯示字元"]
    end
```

---

## 2. Windows 平台：TSF 完整路徑 (windows-chewing-tsf)

### 2.1 程式碼位置清單
- 鍵盤事件轉換：`tip/src/text_service/key_event.rs`
- 核心服務生命週期與分流：`tip/src/text_service/chewing.rs`
- TSF 編輯作業 (Edit Sessions)：`tip/src/text_service/edit_session.rs`
- COM 介面註冊與匯出：`tip/src/text_service/mod.rs`

### 2.2 詳細呼叫鏈 (Call Chain)

1. **事件進入 (Entrypoint)**：
   - 進入點：Windows TSF 框架呼叫 `ITfKeyEventSink::OnTestKeyDown` 與 `ITfKeyEventSink::OnKeyDown`。
   - 轉接函式：`ChewingTextService::on_test_keydown` 與 `ChewingTextService::on_keydown` (`tip/src/text_service/chewing.rs#L516-L525`)。

2. **按鍵轉換 (Key Translation)**：
   - 函式：`SystemKeyboardEvent::to_keyboard_event(self, keymap: KeymapOp)` (`tip/src/text_service/key_event.rs#L69-L103`)。
   - 行為：將 Windows 的 `vk` (Virtual Key)、`scan_code` 及 Shift/Ctrl/Alt 鍵盤狀態封裝為 `chewing::input::KeyboardEvent`。

3. **快捷鍵與模式判斷 (Shortcut & Bypass Gate)**：
   - 檔案：`tip/src/text_service/chewing.rs#L447-L513`
   - 若 `Alt` 按下，直接 bypass（`return Ok(false)`），放行給應用程式。
   - 若 `Ctrl` 按下，非選字/EasySymbol 模式下直接 bypass。
   - 若不在組字狀態（`!self.is_composing()`）且處於英文模式或非可列印字元，直接 bypass。

4. **送交 libchewing 組字 (Engine Processing)**：
   - 函式：`self.chewing_editor.process_keyevent(evt)` (`tip/src/text_service/chewing.rs#L602-L612, L682`)。
   - 引擎返回行為：`let last_behavior = self.chewing_editor.last_key_behavior()` (`#L686`)。
   - 若為 `EditorKeyBehavior::Ignore`，立刻退出放行給系統。

5. **文字提交與 Preedit 更新 (Commit & Composition Flow)**：
   - **情境 A：非組字狀態下的立即提交 (Direct Commit)**
     - 條件：`!self.is_composing() && last_behavior == EditorKeyBehavior::Commit` (`#L694-L706`)。
     - 取得字串：`let text = self.chewing_editor.display_commit().to_owned(); self.chewing_editor.ack();`
     - 提交函式：`self.insert_text(context, &text)` (`#L1046-L1057`)。
     - TSF API：透過非同步編輯會話 `InsertText::new(...)` 呼叫 `ITfRange::SetText` 將 UTF-16 文字插入目標視窗。
   - **情境 B：組字狀態下的提交與 Preedit (In-Composition Commit & Preedit)**
     - 條件：`self.is_composing()`。
     - 取得 commit：`let commit = self.chewing_editor.display_commit().to_owned(); self.chewing_editor.ack();` (`#L715-L720`)。
     - Preedit 函式：`self.update_preedit(context, commit)` (`#L986-L1044`)。
     - 組字字串：若緩衝區有內容，呼叫 `self.set_composition_string(context, &commit, &composition_buf, segments, cursor)`；若為空且無候選字視窗，呼叫 `self.end_composition(context)`。
     - TSF API：透過 `SetCompositionString::new(...)` 或 `EndComposition::new(...)` 調用 `ITfComposition` 與 `ITfContextComposition`。

---

## 3. Linux 平台：Fcitx5 完整路徑 (fcitx5-chewing)

### 3.1 程式碼位置清單
- 輸入法引擎核心：`src/eim.cpp`
- 標頭定義：`src/eim.h`
- 外掛註冊：`src/eim.cpp#L694` (`FCITX_ADDON_FACTORY_V2(chewing, fcitx::ChewingEngineFactory)`)

### 3.2 詳細呼叫鏈 (Call Chain)

1. **事件進入 (Entrypoint)**：
   - 函式：`ChewingEngine::keyEvent(const InputMethodEntry &entry, KeyEvent &keyEvent)` (`src/eim.cpp#L458`)。
   - 過濾放行：若 `keyEvent.isRelease()` 則直接返回。

2. **候選字視窗按鍵攔截 (Candidate Interception)**：
   - 函式：`ChewingEngine::handleCandidateKeyEvent(keyEvent)` (`src/eim.cpp#L386`)。
   - 若觸發選字，執行 `ChewingCandidateWord::select(InputContext *inputContext)` (`#L84-L114`)：
     ```cpp
     chewing_handle_Default(ctx, builtin_selectkeys[...][off]);
     if (chewing_commit_Check(ctx)) {
         inputContext->commitString(safeChewing_commit_String(ctx)); // <--- 提交點 1
     }
     ```

3. **分流與呼叫 libchewing C-API (Key Dispatch to C-ABI)**：
   - 檔案：`src/eim.cpp#L474-L552`
   - 按鍵對應：
     - `FcitxKey_space` -> `chewing_handle_Space(ctx)`
     - `FcitxKey_Tab` -> `chewing_handle_Tab(ctx)`
     - `keyEvent.key().isSimple()` -> 
       - 若有 Shift 且啟用簡易符號：`chewing_set_easySymbolInput(ctx, 1)`
       - 傳送掃描碼：`chewing_handle_Default(ctx, scan_code)` (`#L493`)
     - `FcitxKey_BackSpace` -> `chewing_handle_Backspace(ctx)`
     - `FcitxKey_Escape` -> `chewing_handle_Esc(ctx)`
     - `FcitxKey_Return` / `KP_Enter` -> `chewing_handle_Enter(ctx)`
     - `FcitxKey_space` (with Shift) -> `chewing_handle_ShiftSpace(ctx)`

4. **消化判定與提交 (Absorb & Commit Flow)**：
   - 檔案：`src/eim.cpp#L555-L568`
   - 忽略判定：`if (chewing_keystroke_CheckIgnore(ctx)) return;`（放行由 X11/Wayland 應用程式自行處理）。
   - 吸收判定：`if (chewing_keystroke_CheckAbsorb(ctx)) keyEvent.filterAndAccept();`（IME 吞掉此鍵）。
   - 提交判定：
     ```cpp
     if (chewing_commit_Check(ctx)) {
         keyEvent.filterAndAccept();
         ic->commitString(safeChewing_commit_String(ctx)); // <--- 提交點 2 (關鍵)
     }
     updateUI(ic);
     ```

5. **Preedit 更新與緩衝區沖刷 (Preedit & Flush)**：
   - Preedit 更新：`ChewingEngine::updatePreedit(InputContext *ic)` (`#L644`) -> `updatePreeditImpl(ic)` -> 取得 `safeChewing_buffer_String` 與 `safeChewing_bopomofo_String`，調用 `ic->inputPanel().setClientPreedit(preedit)` 或 `setPreedit`。
   - 焦點切換或重設沖刷：`ChewingEngine::flushBuffer(InputContextEvent &event)` (`#L663-L690`)：
     ```cpp
     if (!text.empty()) {
         event.inputContext()->commitString(text); // <--- 提交點 3 (焦點遺失/切換)
     }
     ```

---

## 4. Linux 平台：IBus 完整路徑 (ibus-chewing)

### 4.1 程式碼位置清單
- 核心引擎實作：`src/ibus-chewing-engine.c`
- 標頭定義：`src/ibus-chewing-engine.h`

### 4.2 詳細呼叫鏈 (Call Chain)
1. **事件進入**：`ibus_chewing_engine_process_key_event(IBusEngine *engine, KSym keySym, guint keycode, guint modifiers)` (`src/ibus-chewing-engine.c#L1016`)。
2. **C-API 處理**：呼叫 `chewing_handle_Default(self->context, ...)`、`chewing_handle_Space(...)` 等。
3. **提交**：若 `chewing_commit_Check(self->context)` 成立，組裝文字並呼叫 `ibus_engine_commit_text(iEngine, self->outgoingText)` (`#L835`)。

---

## 5. 核心引擎：libchewing 內部符號轉換死穴

本專案最重要的源碼發現：**為什麼在現行新酷音半形模式下輸入 `:` 或 `[` 依然會變成全形 `：` 或 `「`？**

### 5.1 關鍵原始碼位置
- 查表定義：`src/conversion/symbol.rs#L7-L53`
- 鍵盤事件分流：`src/editor/mod.rs#L1409-L1456`

### 5.2 核心程式碼追蹤分析

在 `src/editor/mod.rs` 的 `LanguageMode::Chinese` 處理區塊：
```rust
LanguageMode::Chinese => {
    // 1. 若開啟 easy_symbol_input 且按住 Shift，優先查縮寫表
    if shared.options.easy_symbol_input && ev.is_state_on(KeyState::Shift) { ... }

    // 2. 嘗試解析為注音符號
    if !ev.has_modifiers() && KeyBehavior::Absorb == shared.syl.key_press(ev) {
        return self.start_enter_syllable();
    }

    // 3. 【致命點】：在檢查 CharacterForm::Halfwidth 之前，硬編碼調用 special_symbol_input！
    if let Some(symbol) = special_symbol_input(ev.ksym.to_unicode()) {
        shared.com.insert(Symbol::from(symbol));
        shared.snapshot(false, 0);
        return self.spin_absorb();
    }

    // 4. 只有在未命中 special_symbol_input 時，才會走到這裡檢查 Halfwidth/Fullwidth！
    if ev.ksym.is_unicode() {
        match shared.options.character_form {
            CharacterForm::Halfwidth => { ... }
            CharacterForm::Fullwidth => { ... }
        }
    }
}
```

而在 `src/conversion/symbol.rs` 中：
```rust
static SPECIAL_SYMBOLS: [(char, char); 29] = symbol_map! {
    '[' => '「', ']' => '」', '{' => '『', '}' => '』',
    '\'' => '、', '<' => '，', ':' => '：', '\"' => '；',
    '>' => '。', '~' => '～', '!' => '！', '@' => '＠',
    '#' => '＃', '$' => '＄', '%' => '％', '^' => '︿',
    '&' => '＆', '*' => '＊', '(' => '（', ')' => '）',
    '_' => '—', '+' => '＋', '=' => '＝', '\\' => '＼',
    '|' => '｜', '?' => '？', ',' => '，', '.' => '。',
    ';' => '；',
};
```

**結論**：
即使前端設為半形（`Halfwidth`），只要進入 `LanguageMode::Chinese`，上述 29 個字元無一例外均會被 `special_symbol_input` 攔截並轉成全形中文標點存入組字緩衝區！

---

## 6. Kanata 交互與邊界問題 (kanata.original.kbd)

### 6.1 使用者配置要點
- `@capsww`: `(tap-hold 120 180 @caps-dance lctrl)`，短按為 `M-spc`（切換輸入法）或雙擊 Caps Lock。
- `@num`: `(tap-hold 120 180 f (layer-toggle number))`，長按 `f` 進入數字與符號層。
- `@ucln`: `(unicode u+003a)` (冒號 `:`), `@ulb`: `(unicode u+005b)` (中括號 `[`), `@u_`: `(unicode u+005f)` (底線 `_`)。

### 6.2 邊界穿透分析

| Kanata 輸出型態 | OS 攔截狀態 | 輸入法能否看見 | 是否可能觸發全形化 | 責任層級與說明 |
| :--- | :--- | :--- | :--- | :--- |
| `kp1` - `kp0` (數字小鍵盤) | 視為 NumPad 虛擬鍵 | 可以 (VK_NUMPADx / KP_x) | 否 (預設數字) | 正確走數字路徑 |
| 普通按鍵 (如注音按鍵) | 正常 KeyEvent 傳遞 | 可以 | 依輸入法內部狀態 | 進入新酷音注音組字 |
| `(unicode u+003a)` (Windows) | `SendInput` (KEYEVENTF_UNICODE) | **不保證** | 若 TSF 被觸發或應用程式後處理 | 視應用程式是否將其轉為 WM_CHAR 或被 TSF 攔截 |
| `(unicode u+003a)` (Linux) | 模擬 `Ctrl+Shift+U 3 a Enter` | **易衝突** | 是 (若輸入法吞掉 Ctrl+Shift+U) | 輸入法若攔截其一按鍵會造成輸入混亂 |
| 應用程式事後轉換 (如 MS Word) | 文字已 Commit 至應用程式 | **不能阻止** | 是 (由應用程式自己轉為全形) | **超出輸入法控制範圍**，需由應用程式設定停用智慧標點 |

**權限邊界底線**：
1. 輸入法只能控制「經過自己事件迴圈並由自己 commit 的字元」。
2. 若字元直接由 OS 注入至應用程式且未經輸入法，輸入法不保證能攔截。
3. 嚴禁為了攔截應用程式內部行為而建立全域鍵盤 hook 或修改剪貼簿。
