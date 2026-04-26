use crate::state::AppState;
use futures_util::StreamExt;
use lapin::{options::*, types::FieldTable};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct JobPayload {
    client_id: String,
    retrieval_context: String,
    prompt: String,
}

#[derive(Deserialize, Debug)]
struct LLMResponse {
    response: String,
    done: bool,
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
        let state = state.clone();

        tokio::spawn(async move {
            let payload: JobPayload = serde_json::from_slice(&delivery.data).unwrap();
            let client_id = payload.client_id.clone();

            println!("[TASK] Starting inference for :{}", client_id);
            println!("[TASK] Retrieval Context: {:#?}", payload.retrieval_context);

            let _final_prompt = format!(
            "### SYSTEM
            Answer ONLY using the provided context.
            If answer is not present, say: Not found.
            ### CONTEXT
            {}
            ### QUESTION
            {}
            ### ANSWER
            ",
                payload.retrieval_context,
                payload.prompt
            );

            let response = state
                .http_client
                .post(format!("{}/api/generate", state.config.ollama_url))
                .json(&serde_json::json!({
                    "model": "phi3:mini",
                    "prompt": _final_prompt,
                    "stream": true,
                    "options": {
                        "temperature": 0.3,
                        "top_p": 0.9,
                        "top_k": 40,
                        "repeat_penalty": 1.1,
                        "num_ctx": 2048,
                        "num_predict": 200
                    },
                    "stop": ["\n\n"]
                }))
                .send()
                .await;

            match response {
                Ok(res) => {
                    let mut stream = res.bytes_stream();
                    let tx = state.clients.get(&client_id);

                    if let Some(tx) = tx {
                        while let Some(chunk) = stream.next().await {
                            if let Ok(bytes) = chunk {
                                let text = String::from_utf8_lossy(&bytes);

                                for line in text.lines() {
                                    if let Ok(json) = serde_json::from_str::<LLMResponse>(line) {
                                        let _ = tx.send(json.response.clone()).await;
                                        if json.done {
                                            break;
                                        }
                                    } else {
                                        let _ = tx.send(line.to_string()).await;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("Error hitting Ollama: {}", e),
            }

            let _ = delivery.ack(BasicAckOptions::default()).await;
            println!("[Task] Finished for: {}", client_id);
        });
    }
}
