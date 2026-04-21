use axum::{extract::State, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::core::queue::producer::{publish_job, JobPayload};
use crate::modules::ingestion::store::{build_context, embed, query_similar};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ClientPayload {
    client_id: String,
    prompt: String,
}

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClientPayload>,
) -> Json<serde_json::Value> {
    let prompt = &payload.prompt;
    let client_id = &payload.client_id;

    let user_prompt_embedding = embed(&state.http_client, &state.config.ollama_url, prompt).await;

    let similar_chunks = query_similar(&state.http_client, user_prompt_embedding, 5).await;

    let context = build_context(similar_chunks);

    let job = JobPayload {
        client_id: client_id.clone(),
        prompt: prompt.clone(),
        retrieval_context: context,
    };

    publish_job(state.clone(), job).await;

    Json(serde_json::json!({
        "status": "queued"
    }))
}
