#ifndef CHEWING_POLICY_CORE_H
#define CHEWING_POLICY_CORE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    POLICY_PROFILE_DEFAULT = 0,
    POLICY_PROFILE_MARKDOWN = 1,
    POLICY_PROFILE_CODE = 2,
    POLICY_PROFILE_FORMAL = 3,
    POLICY_PROFILE_RAW_COMMAND = 4,
    POLICY_PROFILE_MANUAL_FULLWIDTH = 5
} policy_profile_t;

typedef struct {
    policy_profile_t profile;
    bool has_active_preedit;
    bool is_secure_field;
} policy_context_t;

/**
 * 判斷特定 ASCII 鍵在目前 Profile 與上下文下是否應直接放行給應用程式 (Bypass)
 * 例如在 Markdown/Code 模式且無組字緩衝區時，':', '[', ']' 等鍵回傳 true。
 */
bool policy_should_bypass_key(char ascii_key, const policy_context_t *ctx);

/**
 * 針對 Commit 字串進行策略轉換
 * 若為 ASCII 標點且在 code/markdown profile 則保持半形。
 * 回傳轉換後的 UTF-8 字串指標 (靜態緩衝或有效生命週期)。
 */
const char* policy_transform_commit(const char *utf8_text, const policy_context_t *ctx);

#ifdef __cplusplus
}
#endif

#endif /* CHEWING_POLICY_CORE_H */
