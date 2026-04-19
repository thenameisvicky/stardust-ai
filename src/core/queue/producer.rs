use lapin::{options::BasicPublishOptions, BasicProperties};
use serde::Serialize;

use crate::state::AppState;
use std::sync::Arc;

#[derive(Serialize)]
pub struct JobPayload {
    pub prompt: String,
    pub client_id: String,
    pub retrieval_context: String,
}

pub async fn publish_job(state: Arc<AppState>, payload: JobPayload) {
    let channel = state.amqp.create_channel().await.unwrap();

    let message = serde_json::to_vec(&payload).unwrap();

    channel
        .basic_publish(
            "",
            "LLM_INFERENCE",
            BasicPublishOptions::default(),
            &message,
            BasicProperties::default(),
        )
        .await
        .unwrap();

    println!("Job published to queue");
}
