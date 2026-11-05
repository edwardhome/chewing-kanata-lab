# Windows 原生來源重現與建置

原生來源位於獨立的 `upstream/windows-chewing-tsf` Git 工作目錄，外層以 `docs/patches-windows-tsf/series.json` 與 patch 系列交付。`git am` 會改變提交時間，因此以完整 Git tree 與各 patch 的 SHA-256 驗證內容，不要求重建後提交編號相同。

## 重現來源

需要 Git、PowerShell 與 MSVC Rust 工具鏈。WorkstationX 已實測的 Rust 為 stable 1.99.1；目前沒有證據要求 nightly。以下步驟不安裝輸入法。

```powershell
git clone https://github.com/edwardhome/chewing-kanata-lab.git
cd chewing-kanata-lab
git switch feature/windows-parity-codex
git clone https://codeberg.org/chewing/windows-chewing-tsf.git upstream/windows-chewing-tsf
git -C upstream/windows-chewing-tsf switch -c feature/reproduce-windows 053d31ba10c089ae98508dd4a908ac2c2be3cb49
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/apply_patches.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/apply_patches.ps1 -Check
```

套用前會拒絕未知 HEAD、未提交修改、非開發分支與雜湊不符的 patch。`-Check` 只讀，不套用修補、不建置。不要重設或清除既有工作目錄；另建 worktree 保留本機進度。

## 無介面驗證

```powershell
Push-Location upstream/windows-chewing-tsf
cargo test -p chewing_tip -p chewing_tip_core --lib --locked
cargo test -p tsfreg --locked
cargo build --release -p chewing_tip -p chewing_tip_core -p tsfreg --locked
Pop-Location
cargo test --manifest-path policy-core/Cargo.toml --locked
powershell -NoProfile -ExecutionPolicy Bypass -File packaging/windows/test_package_guard.ps1
```

實際 crate 名稱是 `chewing_tip`。不使用 hosted IME、GUI、自動打字或音效測試。建置與 headless 通過仍不能代替實體按鍵、輸入來源切換與應用相容性驗收。

## 交付界線

早期六份 patch 記錄 Antigravity 的歷史進度，其中仍包含正式新酷音的識別與資料路徑，不能獨立當作安全安裝版本。最新隔離修補與查核結果應一起交付；以 `docs/release/RELEASE_STATUS.md` 與 `docs/verification/runs/` 的實測結果為準。

`packaging/windows/build_package.ps1` 只封裝已提交來源，明列公開字典檔案並產生來源與 SHA-256 manifest。安裝前仍須通過 `do_install.ps1 -ValidateOnly`。不要執行歷史二進位檔的未知 `tsfreg` 參數，舊實作會將未知參數當作解除註冊。

原始 Kanata 配置、正式新酷音與其使用者詞庫保持不動。Windows AI 後端僅允許本機 Ollama；尚未接入驗收的功能不宣稱已完成。

Linux/Fcitx5 adapter patch 為歷史草稿，目前未驗證。macOS AI-Bopomofo 0.6.14 是行為參考，不是本專案的建置或安裝目標。
最後更新：2026-11-05T02:30:16+08:00

1.0.1 的完整設定頁為獨立 Tauri 元件，依 [原生設定頁建置說明](NATIVE_PREFERENCES.md) 準備；不要使用舊 Win32 簡化設定面板。
