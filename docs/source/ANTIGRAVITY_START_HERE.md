# Antigravity 開發啟動卡

## 專案

**新酷音 + Kanata：ASCII-first／全形兼容／跨平台輸入法**

詳細開發研究文件：`Chewing_Kanata_Development_Research.md`  
原始 Kanata 設定：`kanata.original.kbd`（**不可覆寫**）

## 先做這些事

1. 閱讀研究文件全文。確認這是架構研究與原始碼工作交接文件，**不是驗證報告**。
2. 建立 Git 開發分支與 workspace。取得 libchewing、Windows Chewing TSF、Fcitx5 Chewing，上游以 Codeberg 為主；記錄 remote、branch、commit SHA、license。
3. 開始編寫 `UPSTREAM_INVENTORY.md`、`CODE_PATH_MAP.md`、`ARCHITECTURE_DECISIONS.md`，清楚指出 `KeyEvent -> libchewing -> preedit/commit -> application` 每個平台的真正函式路徑。
4. 提出**最小** ASCII-first policy 修改方案：Markdown／程式碼輸入預設保留 ASCII，正式中文書信可選全形／中文標點，原生新酷音可回退，Kanata 原始鍵位不變。
5. 優先 adapter-first，除非研究能證明非改 libchewing 不可。不要先加入 Ollama，也不要使用全域 keyboard hook。
6. 所有實測及正式驗證另立獨立文件，不要混進這份開發研究。

## 特別注意

- Kanata 的 `(unicode u+003a)` 不必然進入 libchewing；原生應用程式事後將 `:` 改為 `：`，也不必然能由 IME 阻止。
- Windows TSF 和 Linux Fcitx5／IBus 需要分別處理平台組字生命週期。
- Vim/Neovim `dd`、`ciw`、`:wq` 不得被 AI 或中文輸入法破壞。
- AI 是後續**可選、非同步、只重排候選字**的功能；目標模型 4GB 記憶體內，但數值需後續另篇實測。
- 不得直接修改使用者現有 Kanata 配置、個人詞庫或系統正式輸入法。

## 第一批預期輸出（全為開發資料，不是測試報告）

- 來源／授權／版本清單
- Windows／Linux 原始碼路徑地圖
- 字元路徑與權限邊界說明
- 符號輸出政策、切換設定草案
- 最小 patch 規劃、回退與上游同步策略
