use axum::{extract::State, Json};
use std::sync::Arc;

use crate::core::queue::producer::{publish_job, JobPayload};
use crate::state::AppState;

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let prompt = payload["prompt"].as_str().unwrap_or("empty prompt");

    let job = JobPayload {
        prompt: prompt.to_string(),
    };

    publish_job(state.clone(), job).await;

    Json(serde_json::json!({
        "status": "queued"
    }))
}
