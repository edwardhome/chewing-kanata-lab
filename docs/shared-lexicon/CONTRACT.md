# Windows 與 Mac 共用個人詞庫契約

狀態：開發中，非已發布功能。基準：AI-Bopomofo main 97006903 與 AI新酷音 1.1.0。

## 分工與資料邊界

Windows 端在 feature/shared-lexicon-v1 開發；Mac 端由 AI-Bopomofo 對話負責。共用的是完成的 UTF-8 JSON 交換檔，不共用運作中的 SQLite、不同步選字分數、不讀取正式新酷音私人詞庫。GIS 只接收挑選的公開程式與合成資料，成果經 Codex 審查，不直接回寫或部署。

## 既有契約

沿用 ai-bopomofo.lexicon / schema_version 1.0.0。保留 zh_phrase、english_term、technical_alias、unicode_mapping 四種類型。entry_id 使用類型、讀音、文字各自 UTF-8 長度的 8-byte big-endian 前綴及 SHA-256。大小寫與 Unicode 原樣保留，不使用 NFKC 或小寫化當身分。

匯入 review 或 approved 都只是待本機確認的資料；來源的 approved、user_confirmed 不能替另一台機器授權部署。匯入缺少既有詞不代表刪除；同音不同字及同字不同讀音均保留。統計可隨檔案保存，但不累加為本機學習次數。

## 必須共同解決的相容缺口

1. Windows v2 工程詞含 zh_aliases，Mac v1 沒有此欄位且拒絕未知欄位。不可把別名塞進 annotation 或 sources，或靜默丟棄。本輪依 Mac 正式交接先完成 v1；擴充另議，不阻擋基本交換。工程匯出預設拒絕省略別名，只有使用者明確同意並看到逐項省略清單，才允許基本欄位匯出，原資料保持不變。
2. Windows 現行工程詞以小寫輸入碼加輸出去重；交換格式身分區分大小寫。交換儲存層必須無損保留，候選查找的去重不可反向修改交換資料。
3. Windows 工程詞目前最多 500 條／256 KiB；Mac 文件最多 20,000 條／8 MiB。不可沿用較小限制截斷共用詞庫，應提供獨立驗證與明確容量錯誤。
4. 中文詞須完整標準注音與聲調，每字一音節。中文別名本身不是注音讀音，不推測或由模型自動補成已核准讀音。
5. 註解、分類、來源及模型建議目前無法全部放入 Windows 工程格式；應以共用格式作獨立持久化層，再產生平台候選投影，而非把交換檔降格成現行工程詞 JSON。

## 驗收與實作順序

- 共用合成測試：字串大小寫、H2O／H₂O、SiO₂、CH₄、多音詞、不同候選、來源審核狀態、重複欄位、非法 ID、未知版本、中文別名與容量邊界。
- Windows 建立獨立 exchange parser/store；匯入顯示新增、重複、衝突及不支援項目，使用者確認後原子儲存與備份。
- 中文詞接入獨立本機候選擴充，工程詞接入既有工程候選；不直接覆寫正式 userphrase.sqlite，不改原生排名。
- 兩邊互相匯出／匯入／再匯出，逐欄比較，不以只驗 JSON 語法宣稱雙平台完成。
- 最終兩個設定頁提供匯入、匯出與回復；私人資料不進公開 Git。自動同步與跨裝置刪除另立契約。

## 目前證據

合成 fixture 位於 tests/shared-lexicon/fixtures/valid-v1.json。check_mac.swift 直接編譯 Mac 現有 LexiconSchema.swift，驗證共用身分與無損 decode/export/decode；它不啟動或安裝輸入法。Windows UI 與候選部署已有開發實作；270 項 Windows 測試通過，7 筆合成資料通過 Windows → Mac → Windows 完整欄位往返。封裝及實機驗收仍待完成，詳見 VALIDATION.md。

## 格式交換與本機部署的差異

7 筆逐欄相等驗證只涵蓋解析／匯出，不表示兩平台核准部署後元資料完全相同。Mac 既有 approve 流程會清除 usage 並更新 review.status；Windows 獨立快照保留來源元資料，再以本機 receipt 授權候選。兩端皆不得把外部核准旗標當作本機授權，不同步或累加內部學習分數。
