# Chewing × Kanata × Context-Aware AI IME
# 獨立驗證計畫與正式發佈 Gate

> 文件代號：CKAI-VERIFY-001  
> 版本：1.0（驗證**計畫**，沒有任何聲稱通過的測試結果）  
> 日期：2026-10-22  
> 與 `00_MASTER_EXECUTION_SPEC.md` **分開**保存；任何測試結果都應記錄在未來真正執行後的 `docs/verification/runs/`。  
> 本文件只定義測試方法、要蒐集的證據、發佈 Gate；**空白不代表通過**。

---

## 1. Status taxonomy（所有平台統一）

- `NOT_RUN`：尚未執行；預設。
- `BLOCKED`：缺原生平台、設備、權限、憑證或上游依賴；要列出原因。
- `PASS`：有原生命令／操作、環境版本、時間、log 或截圖等證據支援。
- `FAIL`：實際重現錯誤並保留 evidence。
- `N/A`：項目本身不適用，需寫原因，**不可用於掩飾沒做**。
- `WAIVED`：非阻斷問題經明確風險承擔者批准；blocker 不可私自 waive。

## 2. 證據紀錄標準

每筆要包含：`run_id`、UTC/Taipei time、OS 及版號、架構、IME/chewing/libchewing SHA、Kanata 版本與原檔 checksum、Neovim 版本、Ollama 及模型 digest（如有）、toolchain、測試步驟、預期結果、實際結果、影片或截圖（**不可包含私人輸入內容**）、log 路徑、判定者及 issue link。

建議紀錄樣本：

```yaml
case_id: CH-ASCII-001
run_id: NOT_RUN
platform: NOT_RUN
build_sha: NOT_RUN
status: NOT_RUN
expected: "ascii_first 模式對來源明確的冒號提交 U+003A"
observed: NOT_RUN
log: null
issue: null
notes: null
```

不要把本 YAML 範例當作真實測試結果。

## 3. 功能回歸矩陣

### A. 基本新酷音功能

| Case | 類別 | 預期行為 |
|---|---|---|
| CH-BASE-001 | 注音組字 | 中文注音可正確編碼與送出 |
| CH-BASE-002 | 候選字 | 上下選字／頁面切換不退化 |
| CH-BASE-003 | 個人詞庫 | 新增／既存使用者詞庫不被破壞 |
| CH-BASE-004 | Backspace／Delete | 游標、刪字、候選與 preedit 一致 |
| CH-BASE-005 | Enter／Space | 正確提交與取消，不多送控制字元 |
| CH-BASE-006 | Profile 切換 | composition 未完成時安全轉移 |
| CH-BASE-007 | 斷線／崩潰 | AI/Bridge 死掉仍可基本輸入 |
| CH-BASE-008 | 原版共存 | 可切換官方 Chewing 而不破壞字典 |

### B. 字元寬度與 Unicode

| Case | 情境 | 必看碼位 |
|---|---|---|
| CH-ASCII-001 | ASCII-first 冒號 | `:` U+003A，而非 `：` U+FF1A |
| CH-ASCII-002 | 左中括號 | `[` U+005B，而非 `［` U+FF3B |
| CH-ASCII-003 | 引號與反引號 | `"` U+0022, `'` U+0027, `` ` `` U+0060 |
| CH-ASCII-004 | Markdown link | `[title](url)` 結構不被全形化 |
| CH-ASCII-005 | code fence | 三個 U+0060 連續保持原樣 |
| CH-ASCII-006 | shell pipe/redirect | `|` U+007C, `>` U+003E, `<` U+003C |
| CH-ASCII-007 | Python 字串／括號 | `()[]{}:,_` 皆正確 |
| CH-ASCII-008 | Email | `name@example.com` 全部 ASCII |
| CH-ASCII-009 | URL | `https://example.org/a?b=c` 全部 ASCII |
| CH-ASCII-010 | 中文句號與逗號 | `，` U+FF0C、`。` U+3002 可明確輸入 |
| CH-ASCII-011 | 中文引號 | `「」` U+300C/U+300D 可明確輸入 |
| CH-ASCII-012 | formal_zh 信件正文 | 中文標點可用，英文字母／Email 不強制全形 |
| CH-ASCII-013 | legacy 回退 | 對照同 SHA 上游結果一致 |
| CH-ASCII-014 | Kanata unicode bypass | 不可由 IME 虛構可控結果；分類正確 |
| CH-ASCII-015 | 應用事後轉換 | 可描述發生在 commit 後，不能假稱 IME 已阻止 |

**注意**：`unicode u+005b` 的實際行為與 Kanata 版本、平台有關，先確立其到底是哪個事件路徑，不要先將期望寫死成「IME 會看到 KeyPress [」。

### C. 中英混打

- 中文→英文→中文連續片語：`我等等call你`、`FPGA 的 Clock`。
- 專業縮寫：`GitHub`、`Python`、`Vivado`、`RTL`、`PCIe`、`AXAU25`。
- 不受誤判破壞：Email、Git Commit、JSON、Markdown code fence、Shell 引號。
- 可以在候選視窗放棄 AI／取消混打，回到原始輸入而不遺失文字。
- 記錄每 100 個混合字詞的輸入法切換次數、選字次數、回退次數、誤提交次數、總按鍵數；採合成測試資料或經同意的測試集。

### D. Vim 與 Terminal

- `dd`、`ciw`、`yy`、`:wq` 在正確 Normal／Cmdline 狀態不被 IME 干擾。
- Insert 模式可輸入中文／英文；Esc 跳出後不留假 composition。
- Visual、Operator pending、Search、Terminal mode 做獨立 case。
- Neovim 正常關閉、crash、server 重啟、切換焦點、雙 Neovim、SSH 遠端 session：過期訊息不能污染另一視窗。
- Wayland 無法取得可信焦點時需 fail-close；不可因猜測而把 raw Vim 指令吃掉。
- VSCodeVim／JetBrains Vim 未開 adapter 時標註未支援，不沿用 Neovim 測試結果。

### E. Ollama 與 AI 安全

- Ollama 未安裝／停止、HTTP 非 200、malformed JSON、非法 candidate_id、重複 ID、返回超時、模型切換，應回原本候選。
- 用者選定候選後到達的 AI 結果必須丟棄；絕不能 retroactively modify。
- 控制同步按鍵路徑沒有 HTTP 呼叫；驗證失效 request、cancel、focus 與 composition revision。
- AI 未經使用者同意不下載模型、不開放 LAN、不得記錄使用者打字；secure field 不呼叫模型。
- 有模型／無模型、0.6B／1.7B 分別測候選重排效果及退化。

## 4. 平台測試矩陣（各自原生環境）

| Platform | 必要 | 重點 |
|---|---|---|
| Windows 11 x64 | v1.0 | 原生 TSF 安裝、UAC、Word、瀏覽器、Terminal、VS Code、Neovim |
| Linux GNOME Wayland + Fcitx5 | v1.0 | GTK、瀏覽器、Terminal、候選窗口、focus |
| Linux KDE Wayland + Fcitx5 | v1.0 | Qt、Terminal、不同應用輸入協定 |
| Linux X11 + Fcitx5 | v1.0 | XIM、GTK、Qt 及混合應用 |
| macOS 實際支援版／真機 | v1.1 | Fcitx5 Chewing、系統輸入來源、權限、Apple app、Ollama |
| Windows 10／ARM64、Linux IBus | Optional | **實際測完才宣稱支援** |

若 CI runner 可建置但不能啟動 GUI，記錄 `BUILD_PASS / GUI_NOT_RUN`，絕不合併成一個 PASS。

## 5. 性能量測與 4 GiB 資源 Gate

### 5.1 環境與測量

- Windows：`nvidia-smi`（若為 NVIDIA GPU）或官方 GPU 工具，記錄 dedicated GPU memory；同時測工作集／Private Bytes。
- Linux：`nvidia-smi`／ROCm 對應工具等；需區分模型、IME、Desktop 與 background load。
- Apple Silicon：使用 Instruments／Activity Monitor／Metal telemetry 等測 unified memory；**不能只拿 Ollama 模型檔案大小當記憶體數據**。
- 記錄 GPU 型號、RAM、模型 tag 與 digest、量化、context window、batch、concurrency、Ollama version、是否有其他模型常駐。

### 5.2 量測指標（目標不是既有結果）

- GPU VRAM 峰值：目標 ≤ 4 GiB（定義為本 IME＋Ollama 推論期間與該模型有關的 GPU 額外佔用；量測方法須固定）。
- Apple unified memory：另建指標與上限報告，不以傳統 discrete VRAM 算法取代。
- Policy 熱路徑新增 p95：目標 ≤ 5 ms；整體按鍵回應和上游基線相較不應有可察覺退化。
- AI rerank timeout 初值：150 ms；計算超時比率、p50、p95、p99、cold start、warm start。
- CPU idle 與 RAM idle、模型關閉後記憶體釋放、開機登入時間、候選視窗抖動。

如果 1.7B 不能達標，測 0.6B；若小模型也干擾輸入則禁用 AI。**資源 Gate 不允許以性能預測或估算代替實測。**

## 6. 隱私與安全 Gate

- 沒有雲端打字輸出、常駐記錄、全域鍵盤 hook、剪貼簿側錄。
- Named Pipe／Unix Socket 不接受跨使用者偽造 session。
- 筆記、log、Issue、截圖沒有個人郵件地址、密碼、真實專案原始碼片段。
- 模型 endpoint `127.0.0.1`，除使用者獨立配置外不能連遠端。
- installer 不會卸載原版新酷音、刪除個人字典或擅自設成系統預設。
- LICENSE／SBOM／模型 license、更新來源與 SHA256 可稽核。

## 7. Install／Upgrade／Rollback／Uninstall Gate

- 乾淨 OS 安裝、已有上游新酷音 OS 安裝、安裝舊版自訂 IME 再升級、降版回復、解除安裝。
- Windows 使用不同 CLSID 與配置 namespace；Linux addon identity 不重疊；macOS bundle／input source identity 不與其他輸入法衝突。
- 升級保留設定與個人字典；卸載只清自己的檔案和註冊、不得破壞系統 keyboard selection。
- 解除安裝後可立即切回原版／系統預設輸入法；必要時需登出登入的 OS，明確告知。
- unsigned／not notarized build 必須明顯標註開發用途；GA 只在合法簽署條件滿足時對一般使用者宣稱正常發佈。

## 8. Gate 定義

### Gate-A：功能開發可併入 integration

`Code SHA + basic build + unit tests + docs + reversible change`；無隱私／資料破壞缺陷。

### Gate-B：Release Candidate

原生 build、installer、最小 GUI smoke、核心注音／Profile／撤銷可用；主要安全政策正常。標 `RC`，未跑的測試必須寫出。

### Gate-C：v1.0 GA（Windows／Linux）

各支援平台矩陣關鍵情境通過、Kanata 不遭更動、AI fallback、原版共存、清楚 Release Notes、無 release blocker，使用者確認任何安裝／簽署權限。

### Gate-D：v1.1 GA（Windows／Linux／macOS）

macOS 真正 GUI 驗證、簽署／公證或明確的可發佈方式、卸載、profile/bridge/AI 功能正常、其他平台無重大回歸。

### Gate-E：維運就緒

上游更新與遷移 SOP、漏洞回報、回退包、相依／license 清單、支援回報格式完整。

## 9. 空白結果模板（Antigravity 在實際運行後填）

```markdown
# RUN-YYYYMMDD-HHMM-platform-build

- Tester / agent:
- Environment:
- Main integration SHA:
- Upstream SHAs:
- Toolchain:
- Kanata version / original file hash:
- Editor version:
- Ollama / model digest:

| Case ID | Status | Actual evidence | Issue |
|---|---|---|---|
| CH-BASE-001 | NOT_RUN | | |

## Performance logs
- Warm latency: NOT_RUN
- Peak GPU memory: NOT_RUN
- Apple unified memory: NOT_RUN

## Release decision
- Decision: NOT_READY
- Blocking reasons:
- Next actions:
```

---

**此文件不構成任何一項驗證成功的證明。只有實際執行後的 run records 才能支持 PASS 或正式發佈。**
