# 公共發行詞庫來源查核報告 (Public Distribution Dictionary Source Audit)

> **查核日期**：2026-11-08  
> **環境**：WorkstationX / Windows 共用工作區  
> **指定查核來源**：`C:\Users\Defu\Documents\AI-Chewing\1.1.0\Dictionary`  
> **查核邊界**：嚴格限定於上述目錄中 6 個具名公開字典檔案及 `package-manifest.json` 所記錄之雜湊，絕未搜尋其他目錄，亦未存取任何使用者個人詞庫或 `%APPDATA%\Chewing` 資料。

---

## 1. 公共發行詞庫目錄與清單 (Public Package Source)

- **確切目錄**：
  `C:\Users\Defu\Documents\AI-Chewing\1.1.0\Dictionary`

- **對照清冊**：
  `C:\Users\Defu\Documents\AI-Chewing\1.1.0\package-manifest.json`

---

## 2. 6 個具名檔案與 Package Manifest 雜湊一致性核對 (Consistency Verification)

經以 SHA-256 演算法重新逐一計算實體檔案內容，並與 `package-manifest.json` 內 `files` 欄位登記之鍵值比對，結果全數**完全一致（Consistent / 100% Match）**：

| 檔案名稱 (`File Name`) | 實體檔案 SHA-256 (`Computed`) | Manifest 登記 SHA-256 (`Expected`) | 一致性 (`Consistency`) |
| :--- | :--- | :--- | :--- |
| `swkb.dat` | `ca57c280dcae602005894515c7e733051ad963184a779116ea37d5ef890277ce` | `ca57c280dcae602005894515c7e733051ad963184a779116ea37d5ef890277ce` | **一致 (MATCH)** |
| `symbols.dat` | `eda621cced1aaf94563a2ec974f61a70e95e31c1577205290cb587bbd1e72b5e` | `eda621cced1aaf94563a2ec974f61a70e95e31c1577205290cb587bbd1e72b5e` | **一致 (MATCH)** |
| `static_dict.bin` | `d24b592cd96f8633cf0a53634d69af39c3a170629a513d76612d45ee23f2ed40` | `d24b592cd96f8633cf0a53634d69af39c3a170629a513d76612d45ee23f2ed40` | **一致 (MATCH)** |
| `rare_dict.bin` | `61a5a97834c4493ba46cca7d0d23ff11efacf74acd646c8d4e3755fa2129efb2` | `61a5a97834c4493ba46cca7d0d23ff11efacf74acd646c8d4e3755fa2129efb2` | **一致 (MATCH)** |
| `static_words.bin` | `969220d5c6ccf658c8fea4965cb4d1c1d991d79a99b387c1264f07216711ef9d` | `969220d5c6ccf658c8fea4965cb4d1c1d991d79a99b387c1264f07216711ef9d` | **一致 (MATCH)** |
| `static_lm.bin` | `cc3e6e14b2e8b41b8663181920e1368e879d7c1ee4733b558824666b8c00728b` | `cc3e6e14b2e8b41b8663181920e1368e879d7c1ee4733b558824666b8c00728b` | **一致 (MATCH)** |

---

## 3. 查核結論
- 指定來源目錄之 6 個具名公開字典檔與 1.1.0 發行清冊清單完全吻合，無任何毀損或不一致。
- 嚴格遵守唯讀與邊界限制：未複製檔案、未安裝、未修改產品源碼、未搜尋其他目錄，無存取任何私人詞庫。
