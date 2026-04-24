use super::store::chunk_with_overlap;
use super::store::embed;
use super::store::store_embedding;
use crate::modules::ingestion::store::build_point;
use crate::state::AppState;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::Arc;

pub async fn ingest_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> (StatusCode, Json<serde_json::Value>) {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let filename = match field.file_name() {
            Some(name) => name.to_string(),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "No filename provided" })),
                );
            }
        };

        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

        if ext != "pdf" && ext != "md" {
            return (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                Json(json!({
                    "error": "Unsupported file type. Only PDF (.pdf) and Markdown (.md) files are accepted."
                })),
            );
        }

        let bytes: axum::body::Bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error reading multipart data: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to read uploaded file" })),
                );
            }
        };

        let text = String::from_utf8_lossy(&bytes).to_string();

        println!("Received file: {}, size: {} bytes", filename, bytes.len());

        let chunks = chunk_with_overlap(&text, 300, 70);

        println!("Generated {} chunks for file: {}", chunks.len(), filename);

        let mut points = Vec::new();

        for chunk in chunks {
            let embedding = embed(&state.http_client, &state.config.ollama_url, &chunk).await;
            println!(
                "Generated embedding for chunk of file {}: {:?}",
                filename,
                embedding.len()
            );

            if let Some(point) = build_point(&chunk, &filename, embedding) {
                points.push(point);
            }
        }

        if let Err(e) = store_embedding(&state.qdrant_client, points).await {
            eprintln!("[ingest_handler] Failed to store embeddings: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to store embeddings in Qdrant. Is Qdrant running?" })),
            );
        }

        return (
            StatusCode::OK,
            Json(json!({
                "status": "received",
                "filename": filename
            })),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "No file uploaded" })),
    )
}
