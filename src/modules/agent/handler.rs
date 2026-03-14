use axum::{extract::State, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::core::queue::producer::{publish_job, JobPayload};
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

    let job = JobPayload {
        client_id: client_id.clone(),
        prompt: prompt.clone(),
    };

    publish_job(state.clone(), job).await;

    Json(serde_json::json!({
        "status": "queued"
    }))
}
