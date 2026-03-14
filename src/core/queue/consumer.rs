use crate::state::AppState;
use futures_util::StreamExt;
use lapin::{options::*, types::FieldTable};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct JobPayload {
    prompt: String,
}

#[derive(Deserialize, Debug)]
struct LLMResponse {
    response: String,
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

        let llm_resp: LLMResponse = serde_json::from_str(&body).unwrap();

        println!("LLM response {:?}", llm_resp);

        if let Err(err) = state.ws_tx.send(llm_resp.response.clone()) {
            eprintln!("Error sending to WS clients: {:?}", err);
        }

        delivery.ack(BasicAckOptions::default()).await.unwrap();
    }
}
