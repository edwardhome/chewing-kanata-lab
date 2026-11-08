# 公共發行詞庫來源查核報告 (Public Distribution Dictionary Source Audit)

> **查核日期**：2026-11-08  
> **環境**：WorkstationX / Windows 共用工作區  
> **查核範圍**：僅限專案及建置發行目錄（Project & Package Build Directories），嚴禁且未存取任何使用者個人詞庫或 `%APPDATA%\Chewing` 資料。

---

## 1. 確切公共發行詞庫目錄 (Exact Public Directory)

- **原始專案發行詞庫目錄**：
  `C:\Users\Defu\Documents\gis-code\chewing-kanata-lab\dist\windows\Dictionary`

- **先前 1.1.0 正式打包成品目錄（對照引用）**：
  `C:\Users\Defu\Documents\AI-Chewing\1.1.0\Dictionary`

上述兩處目錄所包含之 6 項公共發行詞庫檔案完全一致，且完全符合 `packaging/windows/build_package.ps1` 與 `package_guard.ps1` 限定之公開字典清單。

---

## 2. 公共詞庫 6 個檔案 SHA-256 總和校驗碼 (Six SHA-256 Checksums)

| 檔案名稱 (`File Name`) | 檔案大小 (`Bytes`) | SHA-256 校驗碼 (`Lower-case Hex`) |
| :--- | :--- | :--- |
| `swkb.dat` | 154 | `ca57c280dcae602005894515c7e733051ad963184a779116ea37d5ef890277ce` |
| `symbols.dat` | 5,052 | `eda621cced1aaf94563a2ec974f61a70e95e31c1577205290cb587bbd1e72b5e` |
| `static_dict.bin` | 2,797,320 | `d24b592cd96f8633cf0a53634d69af39c3a170629a513d76612d45ee23f2ed40` |
| `rare_dict.bin` | 21,620 | `61a5a97834c4493ba46cca7d0d23ff11efacf74acd646c8d4e3755fa2129efb2` |
| `static_words.bin` | 1,335,378 | `969220d5c6ccf658c8fea4965cb4d1c1d991d79a99b387c1264f07216711ef9d` |
| `static_lm.bin` | 4,863,876 | `cc3e6e14b2e8b41b8663181920e1368e879d7c1ee4733b558824666b8c00728b` |

> [!NOTE]
> 以上 6 個檔案的 SHA-256 與 `C:\Users\Defu\Documents\AI-Chewing\1.1.0\package-manifest.json` 所記錄的發行清單完全吻合。

---

## 3. 安全與唯讀邊界聲明
- 本次查核為純唯讀作業，無執行任何複製、安裝、服務啟動或源碼修改。
- 絕未讀取 `%APPDATA%\Chewing\userphrase.sqlite` 或任何使用者私有詞庫。
