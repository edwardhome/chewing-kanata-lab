#include "ai_reranker.h"
#include <stdio.h>
#include <assert.h>
#include <stdint.h>
#include <string.h>

static void assert_rejected(const rerank_request_t *req) {
    struct {
        rerank_response_t response;
        unsigned char canary[32];
    } guarded;
    memset(&guarded, 0x5a, sizeof(guarded));
    assert(ai_reranker_process(req, &guarded.response) == -1);
    assert(!guarded.response.is_valid);
    assert(guarded.response.ordered_count == 0);
    assert(guarded.response.request_id == NULL);
    assert(!ai_reranker_validate_response(req, &guarded.response));
    for (size_t i = 0; i < sizeof(guarded.canary); ++i) {
        assert(guarded.canary[i] == 0x5a);
    }
}

int main() {
    printf("=========================================\n");
    printf(" AI Reranker Contract & Validation Tests\n");
    printf("=========================================\n");

    candidate_item_t items[3] = {
        {101, "時脈", 0.8},
        {102, "十秒", 0.5},
        {103, "石墨", 0.2}
    };

    rerank_request_t req = {
        .request_id = "req-001",
        .composition_revision = 5,
        .context_type = "code",
        .recent_context = "FPGA 的 ",
        .candidates = items,
        .candidate_count = 3
    };

    // 1. 測試合法重排回應驗證
    rerank_response_t valid_resp = {
        .request_id = "req-001",
        .composition_revision = 5,
        .ordered_candidate_ids = {102, 101, 103},
        .ordered_count = 3,
        .is_valid = true
    };
    assert(ai_reranker_validate_response(&req, &valid_resp) == true);
    printf("  [PASS] Valid candidate reordering accepted\n");

    // 2. 測試非法幻覺 ID (模型自己編造了 999)
    rerank_response_t hallucinated_resp = {
        .request_id = "req-001",
        .composition_revision = 5,
        .ordered_candidate_ids = {101, 999, 103},
        .ordered_count = 3,
        .is_valid = true
    };
    assert(ai_reranker_validate_response(&req, &hallucinated_resp) == false);
    printf("  [PASS] Hallucinated candidate ID strictly rejected\n");

    // 3. 測試重複 ID
    rerank_response_t duplicate_resp = {
        .request_id = "req-001",
        .composition_revision = 5,
        .ordered_candidate_ids = {101, 101, 103},
        .ordered_count = 3,
        .is_valid = true
    };
    assert(ai_reranker_validate_response(&req, &duplicate_resp) == false);
    printf("  [PASS] Duplicate candidate ID strictly rejected\n");

    // 4. 測試離線降級回退
    rerank_response_t fallback_resp;
    int ret = ai_reranker_process(&req, &fallback_resp);
    assert(ret == 0);
    assert(fallback_resp.ordered_count == 3);
    assert(fallback_resp.ordered_candidate_ids[0] == 101);
    assert(fallback_resp.ordered_candidate_ids[1] == 102);
    assert(fallback_resp.ordered_candidate_ids[2] == 103);
    printf("  [PASS] Safe fallback preserves upstream Chewing order\n");

    // 5. 超限輸入在讀取候選前拒絕，無論是否啟用回退。
    rerank_request_t malformed = req;
    malformed.candidate_count = MAX_CANDIDATES_PER_REQUEST + 1;
    assert_rejected(&malformed);
    ai_reranker_init(NULL, NULL, 150);
    assert_rejected(&malformed);
    printf("  [PASS] Oversized request rejected without reading or overwriting arrays\n");

    // 6. size_t 極值也不得進入複製迴圈。
    malformed.candidate_count = SIZE_MAX;
    assert_rejected(&malformed);
    printf("  [PASS] Maximum size_t request rejected\n");

    // 7. 最大合法請求的最後一個候選仍完整保留。
    candidate_item_t boundary[MAX_CANDIDATES_PER_REQUEST];
    for (size_t i = 0; i < MAX_CANDIDATES_PER_REQUEST; ++i) {
        boundary[i] = (candidate_item_t){(int)i, "synthetic", 0.0};
    }
    malformed = req;
    malformed.candidates = boundary;
    malformed.candidate_count = MAX_CANDIDATES_PER_REQUEST;
    assert(ai_reranker_process(&malformed, &fallback_resp) == 0);
    assert(ai_reranker_validate_response(&malformed, &fallback_resp));
    for (size_t i = 0; i < MAX_CANDIDATES_PER_REQUEST; ++i) {
        assert(fallback_resp.ordered_candidate_ids[i] == (int)i);
    }
    printf("  [PASS] Maximum valid request preserves every candidate\n");

    // 8. 空指標與零候選不得留下上一筆有效結果。
    assert_rejected(NULL);
    assert(ai_reranker_process(&req, NULL) == -1);
    assert(!ai_reranker_validate_response(&req, NULL));
    malformed = req;
    malformed.candidates = NULL;
    assert_rejected(&malformed);
    assert(!ai_reranker_validate_response(&malformed, &valid_resp));
    malformed = req;
    malformed.candidate_count = 0;
    assert_rejected(&malformed);
    printf("  [PASS] Null and empty requests invalidate old results\n");

    // 9. 請求自身也不可使用重複 ID。
    candidate_item_t duplicate_items[3] = {items[0], items[0], items[2]};
    malformed = req;
    malformed.candidates = duplicate_items;
    assert_rejected(&malformed);
    assert(!ai_reranker_validate_response(&malformed, &valid_resp));
    printf("  [PASS] Duplicate request IDs rejected\n");

    // 10. 即使候選合法，舊的組字版本也必須丟棄。
    rerank_response_t stale = valid_resp;
    stale.composition_revision -= 1;
    assert(!ai_reranker_validate_response(&req, &stale));
    printf("  [PASS] Stale composition revision rejected\n");

    // 11. 另一筆請求的回覆不可移用。
    stale = valid_resp;
    stale.request_id = "different-request";
    assert(!ai_reranker_validate_response(&req, &stale));
    printf("  [PASS] Different request identity rejected\n");

    // 12. 缺少識別的請求及回覆均不可接受。
    stale.request_id = NULL;
    assert(!ai_reranker_validate_response(&req, &stale));
    malformed = req;
    malformed.request_id = NULL;
    assert_rejected(&malformed);
    malformed.request_id = "";
    assert_rejected(&malformed);
    printf("  [PASS] Missing request identities rejected\n");

    // 13. 回覆宣告超過固定陣列容量時，不得讀越界。
    stale = valid_resp;
    stale.ordered_count = SIZE_MAX;
    assert(!ai_reranker_validate_response(&req, &stale));
    malformed = req;
    malformed.candidate_count = SIZE_MAX;
    assert(!ai_reranker_validate_response(&malformed, &stale));
    printf("  [PASS] Oversized response rejected before array access\n");

    // 14. 已失效或遺漏候選的回覆不得套用。
    stale = valid_resp;
    stale.is_valid = false;
    assert(!ai_reranker_validate_response(&req, &stale));
    stale = valid_resp;
    stale.ordered_count -= 1;
    assert(!ai_reranker_validate_response(&req, &stale));
    printf("  [PASS] Invalid or incomplete responses rejected\n");

    printf("=========================================\n");
    printf(" 14 AI Reranker contract cases passed.\n");
    printf("=========================================\n");
    return 0;
}
