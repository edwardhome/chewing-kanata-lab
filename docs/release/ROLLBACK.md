# 快速回退與安全復原手冊 (Rollback Guide)

> **原則**：任何時候發現輸入行為異常，使用者均可在 10 秒內完全復原至原生新酷音行為。

---

## 1. 軟體級別一鍵回退 (無須重新編譯或重裝)
開啟設定檔：
* Windows: `%APPDATA%\ChewingDev\policy.toml`
* Linux: `~/.config/fcitx5/chewing-policy.toml`

修改內容為：
```toml
default_profile = "legacy"
```
所有按鍵放行 (Bypass) 與字串轉換將立即旁路失效，行為 100% 還原為上游原版新酷音。

---

## 2. Kanata 鍵位安全保證
本專案完全未更動使用者原始之 `kanata.original.kbd`。若需停止 Kanata 改鍵，直接退出 Kanata 背景行程即可恢復物理鍵盤標準配置。

---

## 3. 個人詞庫安全
開發版本詞庫完全隔離儲存於 `%APPDATA%\ChewingDev\`，系統原有之 `%APPDATA%\Chewing\userphrase.sqlite` 保持 100% 原始狀態，零遺失風險。
