/**
 * @file ai_reranker.h
 * @brief Ollama Candidate Reranker Adapter Interface (Local AI)
 * 
 * 嚴格遵循 P08 核心原則：
 * 1. AI 僅重排已存在之合法候選字，不生成任意新文字。
 * 2. 異步非同步執行，逾時 (預設 150ms) 立即放棄，絕不阻塞輸入法按鍵主迴圈。
 * 3. 嚴格白名單檢驗：回傳 ID 必須全部屬於原請求清單，不得重複或缺少。
 */

#ifndef CHEWING_AI_RERANKER_H
#define CHEWING_AI_RERANKER_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MAX_CANDIDATES_PER_REQUEST 16

typedef struct {
    int id;
    const char *text;
    double upstream_score;
} candidate_item_t;

typedef struct {
    const char *request_id;
    int composition_revision;
    const char *context_type;
    const char *recent_context;
    const candidate_item_t *candidates;
    size_t candidate_count;
} rerank_request_t;

typedef struct {
    const char *request_id;
    int composition_revision;
    int ordered_candidate_ids[MAX_CANDIDATES_PER_REQUEST];
    size_t ordered_count;
    bool is_valid;
} rerank_response_t;

/**
 * @brief 初始化本機 Ollama Reranker 配置
 */
void ai_reranker_init(const char *endpoint, const char *model, int timeout_ms);

/**
 * @brief 嚴格白名單校驗函式
 * 檢驗請求識別與組字版本，以及候選 ID 的完整排列；拒絕空清單、超限與重複 ID。
 */
bool ai_reranker_validate_response(
    const rerank_request_t *req,
    const rerank_response_t *resp
);

/**
 * @brief 目前為離線回退實作，原樣保留合法請求的候選順序，並未呼叫模型。
 * 候選數須為 1..MAX_CANDIDATES_PER_REQUEST，ID 不可重複。
 * 不合法請求返回 -1；若 resp 非空則清空並設為無效，呼叫端應保留上游候選。
 */
int ai_reranker_process(
    const rerank_request_t *req,
    rerank_response_t *resp
);

#ifdef __cplusplus
}
#endif

#endif /* CHEWING_AI_RERANKER_H */
