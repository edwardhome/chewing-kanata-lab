# RUN-20261106-AGY-PACKAGE-102

狀態：PASS  
時間：2026-11-06T11:10:00+08:00  
分支：feature/ollama-phrase-prediction  
外層 HEAD：`8af1ec4595c4dd37cf450870f8a00a762e30f6ba`  
原生 TSF：`b5dc92e1fb8f974ed40c79b8bd31907fd2b2dd28`  
偏好設定：`113736a891c24f1ed87f158af715f141fb7162b7`  
作者/提交者：AGY <ckai-dev@local>

---

## 打包任務概述

在 Codex 審查並完成原生 TSF、偏好設定與核心整合建置後，由 AGY 使用獨立乾淨之暫存工作目錄（Worktree），透過 `packaging/windows/build_package.ps1` 執行 `1.0.2-preview.1` 封裝、前置稽核、雜湊校驗與 ZIP 歸檔作業。

---

## 來源與建置成品

- **原生與設定二進位來源**：`%TEMP%\ckai-native-codex-target\release`
  - `chewing_tip.dll` (PE Machine: `0x8664`, x86_64)
  - `tsfreg.exe` (PE Machine: `0x8664`, x86_64)
  - `ChewingPreferences.exe` (PE Machine: `0x8664`, x86_64)
- **詞庫檔案來源**：`C:\Users\Defu\Documents\AI-Chewing\1.0.1\Dictionary`
  - 僅使用六個指定公開發行字典檔（`rare_dict.bin`, `static_dict.bin`, `static_lm.bin`, `static_words.bin`, `swkb.dat`, `symbols.dat`），絕不觸碰使用者個人私有詞庫。
- **上游原始庫指標**：
  - `NativeDirectory`：`upstream/windows-chewing-tsf` (`b5dc92e`)
  - `PreferencesDirectory`：`upstream/windows-chewing-preferences` (`113736a`)

---

## 輸出路徑與成品雜湊

| 成品項目 | 路徑 | SHA-256 雜湊碼 |
| :--- | :--- | :--- |
| **發行目錄** | `C:\Users\Defu\Documents\AI-Chewing\1.0.2-preview.1` | *(目錄)* |
| **發行 ZIP** | `C:\Users\Defu\Documents\AI-Chewing\AI-Chewing-Windows-x64-1.0.2-preview.1.zip` | `a646e86a6f74670519c414552bf4ab9b8691ac94201a7453894b195d31d25dd4` |
| **校驗檔 1** | `C:\Users\Defu\Documents\AI-Chewing\SHA256SUMS-1.0.2-preview.1.txt` | *(包含 ZIP 檔案之 SHA-256)* |
| **校驗檔 2** | `C:\Users\Defu\Documents\AI-Chewing\AI-Chewing-Windows-x64-1.0.2-preview.1.zip.sha256` | *(單檔 sha256 格式)* |

---

## 套件資訊清單（package-manifest.json）

```json
{
    "schema_version": 1,
    "product": "AI-Chewing",
    "version": "1.0.2-preview.1",
    "clsid": "{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}",
    "registry_namespace": "Software\\ChewingDev",
    "data_directory": "ChewingDev",
    "source_commit": "8af1ec4595c4dd37cf450870f8a00a762e30f6ba",
    "windows_source_commit": "b5dc92e1fb8f974ed40c79b8bd31907fd2b2dd28",
    "preferences_source_commit": "113736a891c24f1ed87f158af715f141fb7162b7",
    "preferences_runtime": "WebView2"
}
```

### 完整檔案項目與 SHA-256

| 檔案名稱 | SHA-256 |
| :--- | :--- |
| `chewing_tip.dll` | `0701a6fbf919dc76a41dd1d60d797141eeab3fc0915f5f2cad9445d6d98c66a8` |
| `tsfreg.exe` | `ebebac966576b44866e29f7b2f1700c5f8f6868f22f05a9aaa4842c03c64c51a` |
| `ChewingPreferences.exe` | `5c112de1047d47b525c4a5381a8aaf0bf33601a6440c806662874fcc4107d364` |
| `preferences-GPL-3.0-or-later.txt` | `cde3a5c4c116f8cde7ba83ee7a4a885bc591164195b6123bfcbcdc3a52f9a88c` |
| `preferences-0BSD.txt` | `52464fa7751007fe1fc5f1d94680b2ac83f2839795dde2445fbe3baf94f93885` |
| `preferences-CC-BY-SA-4.0.txt` | `33d3b1685796252291023afe844cbfec8959c76f1875b1c487d0792c7380243b` |
| `preferences-REUSE.toml` | `98f49117036ff4f77aced73ce8971d5197f134583649bd53b02cdedea91c2ae4` |
| `chewing.ico` | `e8a76ea761fb277bb0abead6505fec877dec18f2d8e5228ff9f2d23081ddfeb0` |
| `libchewing-data-CC-BY-4.0.txt` | `4500200b91acb1324538d8e8f9665d5f4142d3f1c5cd9730564cf1895b6170f7` |
| `libchewing-data-NOTICE.txt` | `e50b1d85647ddc7cb30d4c477da759ede77df9b350bc2e7d6257a9391daec7e0` |
| `COPYING.txt` | `230184f60bae2feaf244f10a8bac053c8ff33a183bcc365b4d8b876d2b7f4809` |
| `AI-Bopomofo-LICENSE.txt` | `9cc138135c5fbfba9057868dc3055d154bbc3c936eea5e24d8a9dbb477a7efde` |
| `Dictionary/swkb.dat` | `ca57c280dcae602005894515c7e733051ad963184a779116ea37d5ef890277ce` |
| `Dictionary/symbols.dat` | `eda621cced1aaf94563a2ec974f61a70e95e31c1577205290cb587bbd1e72b5e` |
| `Dictionary/static_dict.bin` | `d24b592cd96f8633cf0a53634d69af39c3a170629a513d76612d45ee23f2ed40` |
| `Dictionary/rare_dict.bin` | `61a5a97834c4493ba46cca7d0d23ff11efacf74acd646c8d4e3755fa2129efb2` |
| `Dictionary/static_words.bin` | `969220d5c6ccf658c8fea4965cb4d1c1d991d79a99b387c1264f07216711ef9d` |
| `Dictionary/static_lm.bin` | `cc3e6e14b2e8b41b8663181920e1368e879d7c1ee4733b558824666b8c00728b` |
| `package_guard.ps1` | `e2c8fdf5991d21a8458135f5a82d458b3e28c97974b3d36386c3cef7ed9e20fb` |
| `do_install.ps1` | `eb8ec483c0ae3be62bbb141f10aba7241e428e3dbef1d3e2d3e0c5f4a09c0655` |
| `do_uninstall.ps1` | `0f124194cc9b5630a6212572982afea8299842d491c2b12163d8872c21c4d640` |
| `install_ime.bat` | `6bfd2eee4adce1c9f397520eab586faa8fae1eac8f99f035355bba16a99bfe6a` |
| `uninstall_ime.bat` | `dacd0bb4dc7c0db81323d776e23299a7f44303950328772b3ff17b75c45d90b1` |
| `version.json` | `52aac7dffd572c0afe844ab2e040557c409870bd2368bb2b00ec7af4f615d297` |

---

## 嚴格驗證結果

### 1. `Assert-ChewingDevPackage` 守衛校驗
- 發行目錄直接稽核：**PASS**
- 解壓縮 ZIP 至暫存目錄稽核：**PASS**
- 隔離 GUID 比對：全數二進位包含獨立 GUID `{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}`，且**完全不含**正式新酷音 GUID `{13F2EF08-575C-4D8C-88E0-F67BB8052B84}`。

### 2. `do_install.ps1 -ValidateOnly` 試跑驗證
```powershell
& 'C:\Users\Defu\Documents\AI-Chewing\1.0.2-preview.1\do_install.ps1' -ValidateOnly
```
- **執行輸出**：`Package preflight passed; no changes made.`
- **退出碼**：`0`
- **安全保障**：未執行任何安裝、COM 註冊、輸入法設定變更、程序終止或權限提升。

### 3. PE 架構檢驗
- `chewing_tip.dll`：PE Offset + 4 Machine = `0x8664`（AMD64 / x86_64）**PASS**
- `tsfreg.exe`：PE Offset + 4 Machine = `0x8664`（AMD64 / x86_64）**PASS**
- `ChewingPreferences.exe`：PE Offset + 4 Machine = `0x8664`（AMD64 / x86_64）**PASS**

### 4. ZIP 封裝完整性
- 歸檔內含 25 個檔案項目，目錄分隔符全數正規化為標準 POSIX 正斜線（`/`），相容於各平台解壓工具。
- 解開之所有檔案 SHA-256 與 `package-manifest.json` 100% 吻合。

---

## 限制與未執行事項

- **未發布、未推送、未合併 main**。
- **未執行實體安裝或註冊**（僅使用 `-ValidateOnly`）。
- **完全保留既有未追蹤報告** `RUN-20261105-OLLAMA-AGY-01.md`。
