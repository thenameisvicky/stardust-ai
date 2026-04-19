use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub async fn create_collection(client: &reqwest::Client) {
    let url = "http://localhost:6333/collections/stardust";

    let body = serde_json::json!({
        "vectors": {
            "size": 768,
            "distance": "Cosine"
        }
    });

    let _ = client.put(url).json(&body).send().await;
}

pub async fn chunk_with_overlap(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let text_len = text.len();

    while start < text_len {
        let end = usize::min(start + chunk_size + overlap, text_len);
        let chunk = text[start..end].to_string();
        chunks.push(chunk);
        start += chunk_size - overlap;
    }

    chunks
}

pub async fn embed(client: &Client, ollama_url: &str, text: &str) -> Vec<f32> {
    let response = client
        .post(format!("{}/api/embed", ollama_url))
        .json(&json!({
            "model": "nomic-embed-text:latest",
            "input": text,
        }))
        .send()
        .await;

    match response {
        Ok(res) => {
            let json: serde_json::Value = res.json().await.unwrap();

            let embedding = json
                .get("embeddings")
                .and_then(|e| e.get(0))
                .and_then(|vec| vec.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect::<Vec<f32>>()
                })
                .unwrap_or_else(|| {
                    println!("Failed to extract embedding vector");
                    Vec::new()
                });

            embedding
        }
        Err(e) => {
            println!("Error hitting Ollama for embedding: {}", e);
            Vec::new()
        }
    }
}

pub async fn store_embedding(client: &Client, embedding: Vec<f32>, text: &str, source: &str) {
    let url = "http://localhost:6333/collections/stardust/points";

    let body = json!({
        "points": [
            {
                "id": Uuid::new_v4().to_string(),
                "vector": embedding,
                "payload": {
                    "text": text,
                    "source": source
                }
            }
        ]
    });

    match client.post(url).json(&body).send().await {
        Ok(_) => println!("Stored vector"),
        Err(e) => println!("Store error: {}", e),
    }
}

pub async fn query_similar(
    client: &Client,
    query_embedding: Vec<f32>,
    top_k: usize,
) -> Vec<(String, f32)> {
    let url = "http://localhost:6333/collections/stardust/points/search";

    let body = json!({
        "vector": query_embedding,
        "limit": top_k,
    });

    let response = client.post(url).json(&body).send().await;

    match response {
        Ok(res) => {
            let json: serde_json::Value = res.json().await.unwrap();

            json["result"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|item| item["payload"]["text"].as_str().map(|s| s.to_string()))
                .collect()
        }
        Err(e) => {
            println!("Search error: {}", e);
            vec![]
        }
    }
}

pub fn build_context(chunks: Vec<String>) -> String {
    chunks.join("\n---\n")
}
