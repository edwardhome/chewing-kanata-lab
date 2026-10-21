#include "ai_reranker.h"
#include <stdio.h>
#include <assert.h>

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

    printf("=========================================\n");
    printf(" All AI Reranker Validation Tests Passed!\n");
    printf("=========================================\n");
    return 0;
}
