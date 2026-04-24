use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, QueryPointsBuilder,
    UpsertPointsBuilder, VectorParamsBuilder,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Payload {
    pub text: String,
    pub source: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Points {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: Payload,
}

pub async fn create_collection(client: &Qdrant) {
    match client.collection_exists("stardust").await {
        Ok(exists) => {
            if !exists {
                println!("Collection 'stardust' not found. Creating...");
                match client
                    .create_collection(
                        CreateCollectionBuilder::new("stardust")
                            .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine)),
                    )
                    .await
                {
                    Ok(_) => println!("Collection 'stardust' created successfully."),
                    Err(e) => eprintln!("Failed to create collection 'stardust': {}", e),
                }
            } else {
                println!("Collection 'stardust' already exists.");
            }
        }
        Err(e) => eprintln!("Error checking if collection exists: {}", e),
    }
}

pub fn chunk_with_overlap(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
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
            let json: serde_json::Value = match res.json().await {
                Ok(v) => v,
                Err(e) => {
                    println!("Embedding parse error: {}", e);
                    return Vec::new();
                }
            };

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

pub fn build_point(text: &str, source: &str, embedding: Vec<f32>) -> Option<Points> {
    if embedding.is_empty() {
        println!("Warning: Empty embedding vector for text: {:?}", text);
        return None;
    }

    Some(Points {
        id: Uuid::new_v4().to_string(),
        vector: embedding,
        payload: Payload {
            text: text.to_string(),
            source: source.to_string(),
        },
    })
}

pub async fn store_embedding(client: &Qdrant, points: Vec<Points>) -> Result<(), String> {
    const BATCH_SIZE: usize = 5;

    for (batch_idx, batch) in points.chunks(BATCH_SIZE).enumerate() {
        let qdrant_points: Vec<PointStruct> = batch
            .iter()
            .map(|p| {
                let payload: HashMap<String, qdrant_client::qdrant::Value> = HashMap::from([
                    ("text".to_string(), p.payload.text.clone().into()),
                    ("source".to_string(), p.payload.source.clone().into()),
                ]);
                PointStruct::new(
                    rand::random::<u64>(),
                    p.vector.clone(),
                    payload,
                )
            })
            .collect();

        client
            .upsert_points(UpsertPointsBuilder::new("stardust", qdrant_points))
            .await
            .map_err(|e| {
                eprintln!("[store_embedding] batch {} upsert failed: {}", batch_idx, e);
                e.to_string()
            })?;

        println!("Stored batch {}/{}", batch_idx + 1, (points.len() + BATCH_SIZE - 1) / BATCH_SIZE);
    }

    Ok(())
}

pub async fn query_similar(
    client: &Qdrant,
    query_embedding: Vec<f32>,
    top_k: usize,
) -> Vec<(String, String, f32)> {
    let result = match client
        .query(
            QueryPointsBuilder::new("stardust")
                .query(query_embedding)
                .limit(top_k as u64)
                .with_payload(true),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[query_similar] Qdrant search failed: {}", e);
            return Vec::new();
        }
    };

    result
        .result
        .into_iter()
        .filter_map(|point| {
            let payload = point.payload;
            let text = payload.get("text")?.as_str()?.to_string();
            let source = payload.get("source")?.as_str()?.to_string();
            let score = point.score;

            Some((text, source, score))
        })
        .collect()
}

pub fn build_context(chunks: Vec<(String, String, f32)>) -> String {
    chunks
        .into_iter()
        .map(|(text, source, score)| {
            format!("[Source: {} | Score: {:.2}]\n{}", source, score, text)
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}
