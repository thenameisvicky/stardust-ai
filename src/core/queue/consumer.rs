use crate::state::AppState;
use futures_util::StreamExt;
use lapin::{options::*, types::FieldTable};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct JobPayload {
    prompt: String,
}

pub async fn run(state: Arc<AppState>) {
    let channel = state.amqp.create_channel().await.unwrap();

    channel
        .queue_declare(
            "LLM_INFERENCE",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let mut consumer = channel
        .basic_consume(
            "LLM_INFERENCE",
            "rust_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    println!("Consumer ready!");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();

        let payload: JobPayload = serde_json::from_slice(&delivery.data).unwrap();

        println!("Received prompt: {}", payload.prompt);

        // TODO: move this later to llm module
        let response = reqwest::Client::new()
            .post("http://localhost:11434/api/generate")
            .json(&serde_json::json!({
                "model": "llama3.2",
                "prompt": payload.prompt,
                "stream": false
            }))
            .send()
            .await
            .unwrap();

        let body = response.text().await.unwrap();

        println!("LLM response {}", body);

        // state.ws_tx.send(body.to_string()).unwrap();

        delivery.ack(BasicAckOptions::default()).await.unwrap();
    }
}
