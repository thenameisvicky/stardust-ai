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

            // let system_prompt = r#"
            //     You are an retrieval-augmented assistant.
            //     Rules:
            //     - Always use the provided context to answer questions.
            //     - If the context does not contain the answer, say "I can't find the information you're looking for"
            //     - Be clear, friendly, and natural.
            //     - Avoid fluff or filler.
            //     - Use common abbreviations where appropriate (AI, CRM, IT, API).
            //     - Prefer concise wording over long explanations.
            //     - Personalize using provided context.
            //     - Do not repeat the prompt.
            //     - Do not add greetings like "Hope you're doing well".
            //     Structure:
            //     Line 1: Context / personalization
            //     Line 2: Problem or insight
            //     Line 3: Suggestion or value
            //     Line 4: Light CTA
            //     Output only the message text, If context is insufficient generate generic text do not invent facts
            // "#;

            let _final_prompt = format!(
                "Answer strictly using the context below.
                Context:{}
                Question:{}
                If not found, say: Not found in any of the documents",
                payload.retrieval_context, payload.prompt
            );

            let response = state
                .http_client
                .post(format!("{}/api/generate", state.config.ollama_url))
                .json(&serde_json::json!({
                    "model": "phi3:mini",
                    "prompt": _final_prompt,
                    "stream": true,
                    "temperature": 0.3,
                    "top_p": 0.9,
                    "top_k": 40,
                    "repeat_penalty": 1.1,
                    "max_tokens": 200,
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
