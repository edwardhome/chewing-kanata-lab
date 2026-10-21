#include "policy_core.h"
#include <string.h>

policy_context_t policy_context_default(void) {
    policy_context_t ctx;
    ctx.mode = POLICY_MODE_LEGACY; /* 預設為 legacy 保證安全回退 */
    ctx.editor_mode = EDITOR_MODE_UNKNOWN;
    ctx.has_active_preedit = false;
    ctx.is_secure_field = false;
    ctx.protect_urls_and_emails = true;
    return ctx;
}

policy_context_t policy_context_create(policy_mode_t mode) {
    policy_context_t ctx = policy_context_default();
    ctx.mode = mode;
    return ctx;
}

bool policy_is_syntax_punctuation(char ch) {
    switch (ch) {
        case '[': case ']':
        case '{': case '}':
        case '(': case ')':
        case ':': case ';':
        case '<': case '>':
        case '`': case '_':
        case '|': case '=':
        case '+': case '-':
        case '*': case '/':
        case '\\': case '~':
        case '^': case '%':
        case '$': case '#':
        case '@': case '!':
        case '?': case '"':
        case '\'':
            return true;
        default:
            return false;
    }
}

bool policy_should_bypass_key(char ascii_key, const policy_context_t *ctx) {
    if (!ctx) {
        return false;
    }

    /* 1. 安全欄位一律放行，保護密碼 */
    if (ctx->is_secure_field) {
        return true;
    }

    /* 2. Vim 非插入模式 (Normal/Visual/Command) 一律放行 */
    if (ctx->editor_mode == EDITOR_MODE_NORMAL ||
        ctx->editor_mode == EDITOR_MODE_VISUAL ||
        ctx->editor_mode == EDITOR_MODE_COMMAND) {
        return true;
    }

    /* 3. Legacy 模式：完全還原上游行為，不主動放行 */
    if (ctx->mode == POLICY_MODE_LEGACY) {
        return false;
    }

    /* 4. Formal ZH 模式：交由輸入法提供中文全形標點 */
    if (ctx->mode == POLICY_MODE_FORMAL_ZH) {
        return false;
    }

    /* 5. ASCII_FIRST 模式：
     * 若組字緩衝區為空 (無未完成之注音)，且當前按鍵為語法標點，
     * 直接放行給應用程式，避免被 libchewing 誤轉為全形！
     */
    if (ctx->mode == POLICY_MODE_ASCII_FIRST) {
        if (!ctx->has_active_preedit && policy_is_syntax_punctuation(ascii_key)) {
            return true;
        }
    }

    return false;
}

/* 映射單一常見誤轉全形符號至 ASCII */
static const char* map_fullwidth_to_ascii(const char *utf8_token) {
    if (strcmp(utf8_token, "\xEF\xBC\x9A") == 0) return ":"; /* ： */
    if (strcmp(utf8_token, "\xE3\x80\x8C") == 0) return "["; /* 「 */
    if (strcmp(utf8_token, "\xE3\x80\x8D") == 0) return "]"; /* 」 */
    if (strcmp(utf8_token, "\xE3\x80\x8E") == 0) return "{"; /* 『 */
    if (strcmp(utf8_token, "\xE3\x80\x8F") == 0) return "}"; /* 』 */
    if (strcmp(utf8_token, "\xEF\xBC\x88") == 0) return "("; /* （ */
    if (strcmp(utf8_token, "\xEF\xBC\x89") == 0) return ")"; /* ） */
    if (strcmp(utf8_token, "\xEF\xBC\x9B") == 0) return ";"; /* ； */
    if (strcmp(utf8_token, "\xE2\x80\x94") == 0) return "_"; /* — (破折號轉回底線) */
    if (strcmp(utf8_token, "\xEF\xBD\x9E") == 0) return "~"; /* ～ */
    if (strcmp(utf8_token, "\xEF\xBC\x81") == 0) return "!"; /* ！ */
    if (strcmp(utf8_token, "\xEF\xBC\x9F") == 0) return "?"; /* ？ */
    return NULL;
}

size_t policy_transform_commit(
    const char *utf8_input,
    const policy_context_t *ctx,
    char *out_buffer,
    size_t out_buffer_size
) {
    if (!utf8_input || !out_buffer || out_buffer_size == 0) {
        return 0;
    }

    size_t in_len = strlen(utf8_input);
    if (in_len == 0) {
        out_buffer[0] = '\0';
        return 0;
    }

    /* 1. 若無上下文或處於 legacy 模式，直接原樣複製 */
    if (!ctx || ctx->mode == POLICY_MODE_LEGACY || ctx->is_secure_field) {
        if (in_len + 1 > out_buffer_size) return 0;
        memcpy(out_buffer, utf8_input, in_len + 1);
        return in_len;
    }

    /* 2. 若處於 formal_zh 模式，維持中文全形標點 */
    if (ctx->mode == POLICY_MODE_FORMAL_ZH) {
        if (in_len + 1 > out_buffer_size) return 0;
        memcpy(out_buffer, utf8_input, in_len + 1);
        return in_len;
    }

    /* 3. 若處於 ascii_first 模式：
     * 檢查是否為單一標點符號的 commit (通常由按鍵直接觸發或選符號提交)
     * 若符合映射表，將其轉回半形 ASCII
     */
    if (ctx->mode == POLICY_MODE_ASCII_FIRST) {
        const char *replacement = map_fullwidth_to_ascii(utf8_input);
        if (replacement != NULL) {
            size_t rep_len = strlen(replacement);
            if (rep_len + 1 > out_buffer_size) return 0;
            memcpy(out_buffer, replacement, rep_len + 1);
            return rep_len;
        }
    }

    /* 其餘一般文字原樣輸出，不破壞中文字詞 */
    if (in_len + 1 > out_buffer_size) return 0;
    memcpy(out_buffer, utf8_input, in_len + 1);
    return in_len;
}

int policy_rerank_candidates_stub(
    const int *candidate_ids,
    size_t candidate_count,
    int *out_ranked_ids
) {
    if (!candidate_ids || !out_ranked_ids || candidate_count == 0) {
        return -1;
    }
    /* Phase 1 Stub: 不改變順序，直接原樣複製 */
    for (size_t i = 0; i < candidate_count; ++i) {
        out_ranked_ids[i] = candidate_ids[i];
    }
    return 0;
}
