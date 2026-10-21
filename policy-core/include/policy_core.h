/**
 * @file policy_core.h
 * @brief Chewing + Kanata 跨平台字元輸出政策核心介面 (C-ABI)
 * 
 * 支援三種核心輸入政策：
 * 1. legacy: 100% 還原上游新酷音原生行為，作為安全回退。
 * 2. ascii_first: 預設保留 ASCII 語法標點（Markdown / 程式碼 / Shell 優先）。
 * 3. formal_zh: 正式繁體中文公文書信標點（「」、，。：；！？等），保護 URL 與 Email。
 */

#ifndef CHEWING_POLICY_CORE_H
#define CHEWING_POLICY_CORE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define POLICY_CORE_VERSION_MAJOR 1
#define POLICY_CORE_VERSION_MINOR 0
#define POLICY_CORE_VERSION_PATCH 0

/**
 * @brief 核心輸出政策模式列舉
 */
typedef enum {
    POLICY_MODE_LEGACY = 0,      /**< 原生新酷音模式 (一鍵回退) */
    POLICY_MODE_ASCII_FIRST = 1, /**< ASCII 標點優先模式 (Markdown / Code 預設) */
    POLICY_MODE_FORMAL_ZH = 2    /**< 正式中文書信模式 (繁體中文標點) */
} policy_mode_t;

/**
 * @brief 編輯器上下文模式 (為 Vim/Neovim Bridge 預留，目前預設 UNKNOWN)
 */
typedef enum {
    EDITOR_MODE_UNKNOWN = 0,
    EDITOR_MODE_NORMAL = 1,
    EDITOR_MODE_INSERT = 2,
    EDITOR_MODE_VISUAL = 3,
    EDITOR_MODE_COMMAND = 4
} editor_mode_t;

/**
 * @brief 政策執行環境上下文
 */
typedef struct {
    policy_mode_t mode;             /**< 當前運行的輸出模式 */
    editor_mode_t editor_mode;      /**< 編輯器回報模式 (預設 UNKNOWN) */
    bool has_active_preedit;        /**< 是否有正在組字中的注音緩衝區 */
    bool is_secure_field;           /**< 是否為密碼或安全欄位 */
    bool protect_urls_and_emails;   /**< 是否主動保護 URL 與 Email */
} policy_context_t;

/**
 * @brief 取得預設的政策上下文 (預設為 LEGACY 模式以保證初始安全回退)
 */
policy_context_t policy_context_default(void);

/**
 * @brief 取得指定的政策上下文
 */
policy_context_t policy_context_create(policy_mode_t mode);

/**
 * @brief 檢查特定字元是否屬於 Markdown / 程式語法受保護標點
 * 涵蓋: [] {} () : ; < > ` _ | = + - * / \ " ' ! @ # $ % ^ & ~ ?
 */
bool policy_is_syntax_punctuation(char ch);

/**
 * @brief 判斷前端是否應在按鍵分流階段將該按鍵 Bypass 放行給應用程式
 * 
 * 核心規則：
 * 1. 若 is_secure_field 為 true，一律放行 (Bypass)。
 * 2. 若 editor_mode 為 NORMAL/VISUAL/COMMAND，一律放行 (保護 Vim 命令)。
 * 3. 若 mode 為 LEGACY，一律由 IME 處理 (不放行)。
 * 4. 若 mode 為 ASCII_FIRST 且無正在進行之注音組字 (has_active_preedit == false)：
 *    若按鍵屬於語法標點，直接放行給應用程式，避免被 libchewing 誤轉為全形。
 * 5. 若 mode 為 FORMAL_ZH，交由 IME 處理以輸出中文標點。
 * 
 * @param ascii_key 輸入之 ASCII 鍵碼 (0x20 - 0x7E)
 * @param ctx 政策上下文
 * @return true 代表放行給應用程式，false 代表由 IME 吸收處理
 */
bool policy_should_bypass_key(char ascii_key, const policy_context_t *ctx);

/**
 * @brief 針對即將 Commit 至應用程式的 UTF-8 字串進行政策過濾與轉換
 * 
 * 核心規則：
 * 1. 若 mode 為 LEGACY，直接原樣輸出。
 * 2. 若 mode 為 ASCII_FIRST：
 *    將誤由符號查表產生之全形標點（如 '：'、'「'、'」'、'（'、'）'）轉回標準半形 ASCII。
 * 3. 若 mode 為 FORMAL_ZH：
 *    保留台灣標準繁體標點（如「」、，。：；！？），但若 context 標註保護且含有 Email/URL 則不替換。
 * 
 * @param utf8_input 原始輸入字串
 * @param ctx 政策上下文
 * @param out_buffer 輸出緩衝區
 * @param out_buffer_size 輸出緩衝區大小 (bytes)
 * @return 寫入輸出緩衝區的長度 (若出錯或 buffer 不足則返回 0)
 */
size_t policy_transform_commit(
    const char *utf8_input,
    const policy_context_t *ctx,
    char *out_buffer,
    size_t out_buffer_size
);

/**
 * @brief AI 候選字重排序 (Reranker) 擴充介面 Stub (Phase 1 預留，不調用模型)
 * 
 * @param candidate_ids 候選字 ID 陣列
 * @param candidate_count 候選字數量
 * @param out_ranked_ids 排序後之 ID 輸出陣列
 * @return 成功回傳 0，失敗或未啟用回傳非零
 */
int policy_rerank_candidates_stub(
    const int *candidate_ids,
    size_t candidate_count,
    int *out_ranked_ids
);

#ifdef __cplusplus
}
#endif

#endif /* CHEWING_POLICY_CORE_H */
