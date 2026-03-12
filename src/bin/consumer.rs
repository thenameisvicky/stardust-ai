use futures_util::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use serde::Deserialize;

#[derive(Deserialize)]
struct JobPayload {
    prompt: String,
}

#[tokio::main]
async fn main() {
    let conn = Connection::connect("amqp://127.0.0.1:5672/%2f", ConnectionProperties::default())
        .await
        .unwrap();

    let channel = conn.create_channel().await.unwrap();

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

        // Call Ollama
        let response = reqwest::Client::new()
            .post("http://localhost:11434/api/generate")
            .json(&serde_json::json!({
                "model": "llama3.2",
                "system": "You are a strict assistant, Respond with exactly one concise sentence, Do not explain or add extra sentences.",
                "prompt": payload.prompt,
                "stream": false,
                "options": {
                    "temprature": 0.2,
                    "num_predict": 40,
                    "top_p": 0.9,
                    "repeat_penalty": 1.1,
                    "stop": ["\n"]
                }
            }))
            .send()
            .await
            .unwrap();

        let body = response.text().await.unwrap();

        let json: serde_json::Value = serde_json::from_str(&body).unwrap();

        let text = json["response"].as_str().unwrap();

        println!("LLM response: {}", text);

        delivery.ack(BasicAckOptions::default()).await.unwrap();
    }
}
