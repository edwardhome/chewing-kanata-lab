# Windows SQLite 交換模組驗證

範圍：僅離線 codec、格式驗證與跨平台 fixture；尚未接入管理 UI、核准部署、回復或實機候選流程，不安裝新功能。

同一 DDL 複製自 Mac schemas/unified-lexicon-v1.sql；輸入 fixture SHA256 30fd51248890dd027719a53f6c1bc5f5a87005aec0459c0ee6968c5c687266f5 已確認。

Rust 以 optional unified-sqlite feature 啟用 rusqlite bundled；預設 TSF 核心不引入 SQLite。外部 bytes 有 16 MiB 上限，複製至私有暫存後唯讀開啟；固定 SQL、版本/schema/完整性/重複欄位/payload/讀音/關聯驗證。外部 approved 仍不是本機核准。Chinese name 保留中文輸出，equivalent 不自動換成符號。

Windows cargo test --locked --offline --features unified-sqlite：168 項通過（138+5+4+9+12），0 失敗；其中12項為新 codec 測試。第一次測試建立非法關聯 fixture 時觸發 SQLite 主鍵衝突，已修正 fixture 更新範圍，重跑全部通過，未隱藏失敗。

Rust 實際 decode/encode 產物：policy-core/tests/fixtures/unified-v1/rust-unified-v1.sqlite 與 rust-unified-v1.json。3 entries、2 equivalent 關聯，JSON 語意與獨立 Python 預期值逐欄相同；SQLite SHA256 69be32a0102357e96c85518bbc9424272f285e3df03d9cb9e259514987c52b89。待 Mac 以 Swift 實際解碼確認，不將 Python/JSON 比較當作 Mac 驗收。

尚待：共同契約最後審查、原生管理儲存層/UI與核准/回復整合、依賴鎖與 nested patch更新、Shift 首字母長按及使用者實機驗收。雲端同步延期，既有 Word 安裝版不變。
