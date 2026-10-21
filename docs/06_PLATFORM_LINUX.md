# Linux 平台輸入法深度研究報告 (Platform Deep Dive: Linux Fcitx5 & IBus)

> **研究對象**：
> - `fcitx5-chewing` (`60096e8ef49df21de6c155d7a0785a58fcccb955` / `5.1.13`)
> - `ibus-chewing` (`6e885f1a8556cb8cfcc657c5b0cfd71b93db4c5c` / `2.2.0`)  
> **目標**：剖析 Linux 環境下 Fcitx5 與 IBus 的按鍵事件迴圈、候選字攔截、Preedit 更新機制、Commit 字串提交點，並確立最小侵入式 Adapter 接點。

---

## 1. Fcitx5 Chewing 內部架構與事件流程 (fcitx5-chewing)

Fcitx5 採用模組化 C++ 設計，`fcitx5-chewing` 作為薄前端外掛，主要邏輯集中於 `src/eim.cpp` 與 `src/eim.h`：

```mermaid
flowchart TD
    subgraph FcitxCore["Fcitx5 Core (Wayland / X11)"]
        KE["fcitx::KeyEvent"]
        IC["fcitx::InputContext"]
        IP["fcitx::InputPanel"]
    end

    subgraph Addon["fcitx5-chewing (src/eim.cpp)"]
        Engine["ChewingEngine::keyEvent"]
        Cand["ChewingEngine::handleCandidateKeyEvent"]
        UI["ChewingEngine::updateUI / updatePreedit"]
        Flush["ChewingEngine::flushBuffer"]
    end

    subgraph CAPI["libchewing C-API"]
        CH_Def["chewing_handle_Default"]
        CH_Spc["chewing_handle_Space"]
        CH_Check["chewing_commit_Check"]
        CH_Str["safeChewing_commit_String"]
    end

    KE --> Engine
    Engine --> Cand
    Engine -->|按鍵分流| CH_Def
    Engine -->|按鍵分流| CH_Spc
    CH_Check -->|true| Engine
    Engine -->|ic->commitString| IC
    Engine --> UI
    UI -->|ic->inputPanel().setPreedit| IP
    Flush -->|焦點切換時 commitString| IC
```

---

## 2. Fcitx5 按鍵分流與提交點深度分析

在 `src/eim.cpp` 中，所有的按鍵處理與字串提交均有明確的進入點與出口：

### 2.1 進入點：`ChewingEngine::keyEvent`
* **位置**：`src/eim.cpp#L458`
* **按鍵類型判定**：
  ```cpp
  if (keyEvent.key().check(FcitxKey_space)) {
      chewingReturnValue = chewing_handle_Space(ctx);
  } else if (keyEvent.key().isSimple()) {
      if (keyEvent.rawKey().states().test(KeyState::Shift)) {
          chewing_set_easySymbolInput(ctx, *config_.EasySymbolInput ? 1 : 0);
      }
      int scan_code = keyEvent.key().sym() & 0xff;
      chewingReturnValue = chewing_handle_Default(ctx, scan_code);
      chewing_set_easySymbolInput(ctx, 0);
  }
  ```
* **按鍵吸收與提交**：
  ```cpp
  if (chewing_keystroke_CheckAbsorb(ctx)) {
      keyEvent.filterAndAccept();
  }
  if (chewing_commit_Check(ctx)) {
      keyEvent.filterAndAccept();
      ic->commitString(safeChewing_commit_String(ctx)); // 【主要提交點】
  }
  updateUI(ic);
  ```

### 2.2 候選字選擇提交點：`ChewingCandidateWord::select`
* **位置**：`src/eim.cpp#L110-L112`
* **邏輯**：當使用者在候選視窗按下選字鍵（例如數字鍵或 Space）時，觸發此處的 `ic->commitString(safeChewing_commit_String(ctx))`。

### 2.3 焦點遺失／緩衝區沖刷：`ChewingEngine::flushBuffer`
* **位置**：`src/eim.cpp#L663-L690`
* **邏輯**：當視窗切換或使用者按快捷鍵切換輸入法時，若設定為 `CommitPreedit`，會呼叫 `chewing_commit_preedit_buf(ctx)` 並由 `event.inputContext()->commitString(text)` 送出未完成的組字文字。

---

## 3. Fcitx5 的最小 Adapter 接點規劃

在 Fcitx5 端，最小修改方案同樣只需修改 `src/eim.cpp`：

### 接點 1：按鍵進入前放行 (Bypass)
在 `ChewingEngine::keyEvent`（約第 473 行）：
```cpp
// 檢查目前是否處於 ASCII-first 模式且組字緩衝區為空
if (isAsciiFirstActive() && chewing_buffer_Check(ctx) == 0 && chewing_bopomofo_Check(ctx) == 0) {
    if (keyEvent.key().isSimple() && !keyEvent.rawKey().states().test(KeyState::Shift)) {
        char keyChar = keyEvent.key().sym() & 0xff;
        if (policy_should_bypass_key(keyChar, &policyContext_)) {
            // 直接 return，不 filterAndAccept，放行給系統/應用程式
            return;
        }
    }
}
```

### 接點 2：Commit 字串過濾
將 `safeChewing_commit_String(ctx)` 產生的字串透過 `policy_transform_commit` 處理後再傳遞給 `ic->commitString()`。

---

## 4. IBus 前端架構速覽 (ibus-chewing)

IBus-Chewing 採用 GObject / C23 架構，建置系統為 Meson：
* **核心檔案**：`src/ibus-chewing-engine.c`
* **按鍵進入**：`ibus_chewing_engine_process_key_event` (`#L1016`)。
* **提交點**：`ibus_engine_commit_text(iEngine, self->outgoingText)` (`#L835`)。
* **評估結論**：Fcitx5 在現代 Linux 桌面（尤其是 Wayland 與繁體中文社群）為主流首選；IBus 前端架構相仿，共用之 `policy-core` (C-ABI) 可完全套用於 IBus。
