#include "ai_reranker.h"
#include <string.h>

static char g_endpoint[128] = "http://127.0.0.1:11434";
static char g_model[64] = "qwen3:0.6b";
static int g_timeout_ms = 150;
static bool g_enabled = false;

void ai_reranker_init(const char *endpoint, const char *model, int timeout_ms) {
    if (endpoint) {
        strncpy(g_endpoint, endpoint, sizeof(g_endpoint) - 1);
        g_endpoint[sizeof(g_endpoint) - 1] = '\0';
    }
    if (model) {
        strncpy(g_model, model, sizeof(g_model) - 1);
        g_model[sizeof(g_model) - 1] = '\0';
    }
    if (timeout_ms > 0) {
        g_timeout_ms = timeout_ms;
    }
    g_enabled = true;
}

/* Reject malformed requests before reading a candidate or writing a result. */
static bool request_is_valid(const rerank_request_t *req) {
    if (!req || !req->request_id || !req->request_id[0] || !req->candidates ||
        req->candidate_count == 0 ||
        req->candidate_count > MAX_CANDIDATES_PER_REQUEST) {
        return false;
    }
    for (size_t i = 0; i < req->candidate_count; ++i) {
        for (size_t j = 0; j < i; ++j) {
            if (req->candidates[i].id == req->candidates[j].id) {
                return false;
            }
        }
    }
    return true;
}

bool ai_reranker_validate_response(
    const rerank_request_t *req,
    const rerank_response_t *resp
) {
    if (!request_is_valid(req) || !resp || !resp->is_valid ||
        !resp->request_id || strcmp(req->request_id, resp->request_id) != 0 ||
        req->composition_revision != resp->composition_revision) {
        return false;
    }

    if (resp->ordered_count != req->candidate_count) {
        return false; /* 數量不符 */
    }

    /* 檢查每個 ID 是否均來自原始候選清單，且無重複 */
    bool used[MAX_CANDIDATES_PER_REQUEST] = {false};

    for (size_t i = 0; i < resp->ordered_count; ++i) {
        int target_id = resp->ordered_candidate_ids[i];
        bool found = false;

        for (size_t j = 0; j < req->candidate_count; ++j) {
            if (req->candidates[j].id == target_id) {
                if (used[j]) {
                    return false; /* 重複 ID */
                }
                used[j] = true;
                found = true;
                break;
            }
        }

        if (!found) {
            return false; /* 包含非請求內的未知 ID (嚴禁偽造候選) */
        }
    }

    return true;
}

int ai_reranker_process(
    const rerank_request_t *req,
    rerank_response_t *resp
) {
    if (!resp) {
        return -1;
    }

    /* A failed call must not leave a previously valid suggestion reusable. */
    memset(resp, 0, sizeof(*resp));
    if (!request_is_valid(req)) {
        return -1;
    }

    resp->request_id = req->request_id;
    resp->composition_revision = req->composition_revision;

    /* 
     * 在 Phase 1 / 本地無 Ollama 守護進程時，
     * 執行確定性的詞頻啟發式或維持原版新酷音順序，
     * 確保不因 AI 服務停止而卡住打字。
     */
    resp->ordered_count = req->candidate_count;
    for (size_t i = 0; i < req->candidate_count; ++i) {
        resp->ordered_candidate_ids[i] = req->candidates[i].id;
    }
    resp->is_valid = true;

    return 0;
}
