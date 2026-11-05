# AI 候選回退與 Windows 套件前置檢查

更新時間：2026-11-05T01:54:11+08:00

- 執行者：Codex
- 平台：macOS 27.0.1，arm64；Apple clang 21.0.0。
- 工作分支：`feature/windows-parity-codex`，基準 `2ce7a514c7531aaa085502321f10f76184e8e7f6`。
- PowerShell：7.5.4，官方 macOS arm64 可攜版，在臨時目錄執行，沒有全域安裝。
- 無 IME、GUI、音訊、正式登錄或私人詞庫操作。

## 本次執行結果

| 檢查 | 狀態 | 實際結果 |
| --- | --- | --- |
| C 候選契約 | PASS | 14 個案例通過，啟用 AddressSanitizer 與 UndefinedBehaviorSanitizer |
| PowerShell 套件前置檢查 | PASS | 16 個案例通過，使用合成套件與臨時目錄 |
| 本輪總計 | PASS | 30 個案例通過，0 失敗、0 跳過 |
| Windows 原生 DLL 編譯 | NOT_RUN | 尚未取得遠端最新工作樹進行編譯 |
| Windows PowerShell 5.1、實際 TSF 安裝／解除安裝／回退 | NOT_RUN | macOS 的前置檢查不代表 Windows 登錄或 COM 操作通過 |
| Ollama 與 IME 實鍵驗收 | NOT_RUN | 此輪沒有模型推論或實際打字 |

C 案例包含最大合法候選數、超過陣列容量、SIZE_MAX、空指標、空清單、重複請求 ID、過期組字版本、不同請求、遺漏候選與保留原候選順序。非法請求清除舊的有效狀態，不會嘗試把超限清單複製進固定陣列。

PowerShell 案例包含缺少 manifest、正式新酷音 GUID／偏好／資料路徑、路徑穿越、必需檔案缺漏、摘要不符、宣稱隔離但二進位仍帶正式 GUID、缺少隔離 GUID、版本路徑注入、未固定來源、詞庫資源、ValidateOnly 不執行合成 EXE／不建立安裝目錄，以及所有套件腳本的語法解析。

## 重現命令

```sh
clang -std=c99 -Wall -Wextra -Werror -fsanitize=address,undefined -g \
  -Iai-adapter/include ai-adapter/src/ai_reranker.c \
  ai-adapter/tests/test_ai_reranker.c -o /tmp/test_ai_reranker
/tmp/test_ai_reranker
pwsh -NoLogo -NoProfile -File packaging/windows/test_package_guard.ps1
```

PowerShell 來源：[官方 7.5.4 附件與摘要](https://github.com/PowerShell/PowerShell/releases/tag/v7.5.4)。下載的 `powershell-7.5.4-osx-arm64.tar.gz` 已核對 SHA-256：`3aaadd7ca62f1e4dbe59145b6af24e926d61f8da8a4782bc535e500c184135f0`。

## 尚未完成

Windows 當前程式碼仍需要全面隔離 GUID、偏好與詞庫路徑，再以原生建置產生真實套件 manifest。新安裝器會拒絕缺少 manifest 的舊套件；這是故意阻止未完成隔離的二進位進入註冊，不可用手工填寫 manifest 代替程式碼稽核與建置驗證。

二進位 GUID 與 SHA-256 檢查不能證明所有執行路徑安全，也不能取代原生來源審查、安裝／解除安裝測試或使用者實鍵驗收。此提交不構成可安裝版本，未發布套件。
