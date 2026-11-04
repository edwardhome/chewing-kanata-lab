# Windows TSF 可重現建置指南

> 文件版本：CKAI-BOOTSTRAP-002  
> 基準上游 SHA：`053d31ba10c089ae98508dd4a908ac2c2be3cb49`  
> 本地 HEAD：`2232ff9b91efae8247842eb0d34b8880c30d8e50`  
> AI-Bopomofo 參考版本：0.6.14 commit `9700690321df88`

---

## 環境需求

| 工具 | 版本需求 | 驗證指令 |
|------|---------|---------|
| Rust (stable) | ≥ 1.85 (nightly for unstable prefs) | `rustup show` |
| Cargo | ≥ 1.85 | `cargo --version` |
| MSVC Build Tools | 2022 (v143) | `cl.exe` |
| Windows SDK | 10.0.26100 以上 | `winver` |
| Git | ≥ 2.40 | `git --version` |
| PowerShell | ≥ 7.3 | `$PSVersionTable.PSVersion` |

> [!IMPORTANT]
> 本專案使用 `nightly` toolchain 建置 `chewing_preferences` crate。  
> `tip` 與 `crates/chewing_tip_host` 使用穩定 toolchain。  
> 執行 `rustup show` 確認兩者均已安裝。

---

## 1. 從乾淨環境重現步驟

```powershell
# 1. 克隆外層 repo（不含 upstream/）
git clone https://github.com/你的帳號/chewing-kanata-lab.git
cd chewing-kanata-lab

# 2. 克隆真正的 upstream（獨立位置，不納入外層 git）
git clone https://codeberg.org/chewing/windows-chewing-tsf.git upstream\windows-chewing-tsf
cd upstream\windows-chewing-tsf

# 3. 切到已知基準 commit（上游最後一個乾淨 commit）
git checkout 053d31ba10c089ae98508dd4a908ac2c2be3cb49

# 4. 套用 6 個 patch（來自 docs/patches-windows-tsf/）
$patches = Get-ChildItem ..\..\docs\patches-windows-tsf\*.patch | Sort-Object Name
foreach ($p in $patches) {
    Write-Host "Applying: $($p.Name)"
    git am --3way $p.FullName
    if ($LASTEXITCODE -ne 0) {
        Write-Error "FAILED on $($p.Name). Run: git am --abort"
        exit 1
    }
}

# 5. 驗證 HEAD 與預期 SHA 一致
$expected = "2232ff9b91efae8247842eb0d34b8880c30d8e50"
$actual = git rev-parse HEAD
if ($actual -ne $expected) {
    Write-Error "SHA mismatch! Expected: $expected Got: $actual"
    exit 1
}
Write-Host "OK: HEAD = $actual"

# 6. 建置（headless，不安裝）
cd upstream\windows-chewing-tsf
cargo build --workspace 2>&1
if ($LASTEXITCODE -ne 0) { exit 1 }

# 7. 執行無介面測試
cargo test --workspace --lib -- --test-output immediate
```

---

## 2. Patch 系列說明

位於 `docs/patches-windows-tsf/`，以 `git format-patch` 格式匯出：

| 編號 | 檔案 | SHA (全) | 內容摘要 |
|------|------|----------|---------|
| 0001 | `0001-feat-tip-embed-Kanata-number-layer...patch` | `9b3223c5cd37b2de9b0267f466f6a0ddaf6efd38` | 嵌入 Kanata number layer、修正選單啟動 |
| 0002 | `0002-feat-tip-integrate-local-Ollama...patch` | `d434747e0e9dc0d57f9686108e2affb7986954ce` | 整合本機 Ollama AI 用戶端與語法糾正 |
| 0003 | `0003-fix-tip-add-sandbox-detection...patch` | `fe689db9cc4e1e365a83e78a60b7e7b00458b618` | 沙盒偵測守護與 mutex 同步 |
| 0004 | `0004-feat-kanata-implement-AI-Bopomofo-v2...patch` | `5e35402e76db9469149abd25aefbec6d4b2fb991` | F 分層延遲控制、Shift 旁路、標點保留、偏好設定 |
| 0005 | `0005-fix-menu-prevent-menu-crash...patch` | `6a62bd72ea916520808761d709a1138d4064d7e3` | 選單 crash 修正、TSF profile 強制登記 |
| 0006 | `0006-fix-tsf-fix-preferences-freeze...patch` | `2232ff9b91efae8247842eb0d34b8880c30d8e50` | 偏好設定凍結、英文模式 F 分層、Shift 英文連續輸入 |

> [!CAUTION]
> 嚴禁使用 `git reset --hard`、`git clean`、`git push`、`rustup override set` (全域)。  
> 所有建置在 `upstream/windows-chewing-tsf/target/` 本地快取，不影響系統安裝。

---

## 3. 已知已修正問題（原 adapter_tsf.patch）

原 `adapters/windows-tsf/adapter_tsf.patch` **已棄用**，包含以下錯誤：

| 問題 | 錯誤符號 | 正確符號 |
|------|---------|---------|
| 枚舉值不存在 | `PolicyProfile::Markdown` | `PolicyMode::AsciiFirst` |
| Rust API 簽名錯誤 | `transform_commit(&commit, &ctx).to_owned()` → arity mismatch | 實際 API: `transform_commit(&commit, &ctx) -> Cow<str>` ✓ (已在 chewing.rs 中正確使用) |
| C API 名稱錯誤 | `policy_transform_commit(raw, &ctx)` (3 args) | `policy_transform_commit(input, ctx, buf, buf_size)` (4 args) |

原 `adapters/fcitx5/adapter_fcitx5.patch` **已棄用**，包含以下錯誤：

| 問題 | 錯誤符號 | 正確符號 |
|------|---------|---------|
| C enum 名稱錯誤 | `POLICY_PROFILE_MARKDOWN` | `POLICY_MODE_ASCII_FIRST` |
| C API arity 錯誤 | `policy_transform_commit(str, ctx)` | `policy_transform_commit(str, ctx, buf, size)` |

> [!NOTE]
> `tip/src/text_service/chewing.rs` 中的 Rust 整合已使用正確 API (`PolicyMode::AsciiFirst`, `transform_commit(&commit, &ctx)`)。  
> C adapter patches 為廢棄歷史文件，需重寫才能使用。Linux/Fcitx5 路徑已標記為 UNSUPPORTED (見下)。

---

## 4. 支援狀態矩陣

| 平台 | 建置路徑 | 狀態 |
|------|---------|------|
| **Windows TSF** (tip crate) | `cargo build -p tip` | ✅ SUPPORTED |
| Windows preferences | `cargo build -p chewing_preferences` (nightly) | ✅ SUPPORTED |
| Linux Fcitx5 | `adapters/fcitx5/adapter_fcitx5.patch` | ❌ STALE — patch 有錯誤，需重寫 |
| Linux IBus | 無 adapter | ❌ NOT_STARTED |
| macOS | 無 (Apple Foundation Models 不可用) | ❌ OUT_OF_SCOPE |

---

## 5. Cargo 依賴鎖定

`upstream/windows-chewing-tsf/Cargo.lock` 已納入版本控制。  
`[patch.crates-io]` 中的 `chewing` 指向 `https://codeberg.org/chewing/libchewing.git`，  
如網路受限，改用 `# chewing = { path = "../libchewing/" }` 並手動 clone libchewing。

驗證 Cargo.lock SHA256：
```powershell
Get-FileHash upstream\windows-chewing-tsf\Cargo.lock -Algorithm SHA256 | Select-Object Hash
```

---

## 6. 外層 repo 忽略說明

`chewing-kanata-lab/.gitignore` 明確忽略 `upstream/`，原因：

1. upstream 是獨立 Git repo，不能巢狀跟蹤
2. patch series (`docs/patches-windows-tsf/`) 是唯一可重現的 source delivery
3. 不能在外層 `git add upstream/` 以避免破壞上游 git history

**修補套用腳本**位於 `scripts/apply_patches.ps1`。

---

_最後更新：2026-11-05 (feature/ai-bopomofo-parity Stage A)_
