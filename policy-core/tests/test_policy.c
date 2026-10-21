#include "policy_core.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

static int g_tests_run = 0;
static int g_tests_passed = 0;

#define TEST_ASSERT(cond, msg) do { \
    g_tests_run++; \
    if (cond) { \
        g_tests_passed++; \
        printf("  [PASS] %s\n", msg); \
    } else { \
        printf("  [FAIL] %s (line %d)\n", msg, __LINE__); \
    } \
} while (0)

void test_default_context(void) {
    printf("Running test_default_context...\n");
    policy_context_t ctx = policy_context_default();
    TEST_ASSERT(ctx.mode == POLICY_MODE_LEGACY, "Default mode must be LEGACY for safe fallback");
    TEST_ASSERT(ctx.has_active_preedit == false, "Default preedit must be false");
    TEST_ASSERT(ctx.is_secure_field == false, "Default is_secure_field must be false");
    TEST_ASSERT(ctx.editor_mode == EDITOR_MODE_UNKNOWN, "Default editor mode must be UNKNOWN");
}

void test_ascii_first_bypass(void) {
    printf("Running test_ascii_first_bypass...\n");
    policy_context_t ctx = policy_context_create(POLICY_MODE_ASCII_FIRST);
    ctx.has_active_preedit = false;

    /* 語法符號在無組字緩衝區時應放行 */
    TEST_ASSERT(policy_should_bypass_key(':', &ctx) == true, "Colon ':' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key('[', &ctx) == true, "Left bracket '[' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key(']', &ctx) == true, "Right bracket ']' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key('{', &ctx) == true, "Left brace '{' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key('}', &ctx) == true, "Right brace '}' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key('(', &ctx) == true, "Left paren '(' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key(')', &ctx) == true, "Right paren ')' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key('`', &ctx) == true, "Backtick '`' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key('_', &ctx) == true, "Underscore '_' should bypass in ASCII-first");
    TEST_ASSERT(policy_should_bypass_key(';', &ctx) == true, "Semicolon ';' should bypass in ASCII-first");

    /* 當有正在組字注音時，不應放行，優先保護中文組字 */
    ctx.has_active_preedit = true;
    TEST_ASSERT(policy_should_bypass_key(':', &ctx) == false, "Colon ':' must NOT bypass when preedit is active");
    TEST_ASSERT(policy_should_bypass_key('[', &ctx) == false, "Left bracket '[' must NOT bypass when preedit is active");
}

void test_legacy_mode_fallback(void) {
    printf("Running test_legacy_mode_fallback...\n");
    policy_context_t ctx = policy_context_create(POLICY_MODE_LEGACY);
    ctx.has_active_preedit = false;

    /* Legacy 模式下一律不放行，完全維持原版新酷音行為 */
    TEST_ASSERT(policy_should_bypass_key(':', &ctx) == false, "Colon must NOT bypass in legacy mode");
    TEST_ASSERT(policy_should_bypass_key('[', &ctx) == false, "Bracket must NOT bypass in legacy mode");

    /* Commit 轉換亦原樣輸出 */
    char out[64];
    size_t len = policy_transform_commit("\xEF\xBC\x9A", &ctx, out, sizeof(out)); /* ： */
    TEST_ASSERT(len > 0 && strcmp(out, "\xEF\xBC\x9A") == 0, "Legacy mode must keep fullwidth colon untouched");
}

void test_formal_zh_mode(void) {
    printf("Running test_formal_zh_mode...\n");
    policy_context_t ctx = policy_context_create(POLICY_MODE_FORMAL_ZH);
    ctx.has_active_preedit = false;

    /* Formal ZH 模式下一律由 IME 處理，以產生全形標點 */
    TEST_ASSERT(policy_should_bypass_key(':', &ctx) == false, "Formal mode should not bypass key");

    /* Commit 轉換保持中文全形標點 */
    char out[64];
    size_t len = policy_transform_commit("\xEF\xBC\x9A", &ctx, out, sizeof(out)); /* ： */
    TEST_ASSERT(len > 0 && strcmp(out, "\xEF\xBC\x9A") == 0, "Formal mode must preserve fullwidth colon");

    len = policy_transform_commit("\xE3\x80\x8C", &ctx, out, sizeof(out)); /* 「 */
    TEST_ASSERT(len > 0 && strcmp(out, "\xE3\x80\x8C") == 0, "Formal mode must preserve opening quote");
}

void test_secure_and_vim_modes(void) {
    printf("Running test_secure_and_vim_modes...\n");
    policy_context_t ctx = policy_context_create(POLICY_MODE_LEGACY);
    ctx.is_secure_field = true;

    /* 安全欄位一律放行 */
    TEST_ASSERT(policy_should_bypass_key(':', &ctx) == true, "Secure field must bypass all keys");

    /* Vim Normal / Command 模式一律放行 */
    ctx.is_secure_field = false;
    ctx.editor_mode = EDITOR_MODE_NORMAL;
    TEST_ASSERT(policy_should_bypass_key('d', &ctx) == true, "Vim Normal mode must bypass 'd'");
    TEST_ASSERT(policy_should_bypass_key(':', &ctx) == true, "Vim Normal mode must bypass ':'");

    ctx.editor_mode = EDITOR_MODE_COMMAND;
    TEST_ASSERT(policy_should_bypass_key('w', &ctx) == true, "Vim Command mode must bypass 'w'");
}

void test_transform_commit(void) {
    printf("Running test_transform_commit...\n");
    policy_context_t ctx = policy_context_create(POLICY_MODE_ASCII_FIRST);
    char out[64];

    /* 測試符號轉換回半形 ASCII */
    size_t len = policy_transform_commit("\xEF\xBC\x9A", &ctx, out, sizeof(out)); /* ： */
    TEST_ASSERT(len == 1 && strcmp(out, ":") == 0, "Transform '：' to ':' in ascii_first mode");

    len = policy_transform_commit("\xE3\x80\x8C", &ctx, out, sizeof(out)); /* 「 */
    TEST_ASSERT(len == 1 && strcmp(out, "[") == 0, "Transform '「' to '[' in ascii_first mode");

    len = policy_transform_commit("\xE3\x80\x8D", &ctx, out, sizeof(out)); /* 」 */
    TEST_ASSERT(len == 1 && strcmp(out, "]") == 0, "Transform '」' to ']' in ascii_first mode");

    /* 測試中文字詞絕對不被破壞 */
    const char *chinese_word = "新酷音輸入法測試";
    len = policy_transform_commit(chinese_word, &ctx, out, sizeof(out));
    TEST_ASSERT(len == strlen(chinese_word) && strcmp(out, chinese_word) == 0, "Chinese phrases must remain 100% untouched");
}

void test_reranker_stub(void) {
    printf("Running test_reranker_stub...\n");
    int input_ids[] = {101, 102, 103, 104};
    int output_ids[4] = {0};
    int res = policy_rerank_candidates_stub(input_ids, 4, output_ids);
    TEST_ASSERT(res == 0, "Reranker stub should return success (0)");
    TEST_ASSERT(output_ids[0] == 101 && output_ids[1] == 102 && output_ids[2] == 103 && output_ids[3] == 104, 
                "Reranker stub must preserve candidate order");
}

int main(void) {
    printf("=========================================\n");
    printf(" Chewing Policy Core Unit Test Suite\n");
    printf("=========================================\n");

    test_default_context();
    test_ascii_first_bypass();
    test_legacy_mode_fallback();
    test_formal_zh_mode();
    test_secure_and_vim_modes();
    test_transform_commit();
    test_reranker_stub();

    printf("=========================================\n");
    printf(" Test Results: %d / %d passed.\n", g_tests_passed, g_tests_run);
    printf("=========================================\n");

    return (g_tests_passed == g_tests_run) ? 0 : 1;
}
