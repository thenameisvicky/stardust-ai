use axum::{routing::post, Json, Router, extract::State};
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties, BasicProperties};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AiRequest {
    prompt: String,
    client_id: String,
}

struct AppState {
    amqp_conn: Connection,
}

// Tokio for parallel processing the incoming traffic
// Automatically spawn threads using CPU cores
#[tokio::main]
async fn main() {
    let addr = "amqp://127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await.unwrap();
    let channel = conn.create_channel().await.unwrap();

    channel.queue_declare("wai_prompts", QueueDeclareOptions::default(), FieldTable::default()).await.unwrap();

    //Share the one time resource created across the routes
    let shared_state = Arc::new(AppState { amqp_conn: conn });

    //API route
    let app = Router::new()
        .route("/generate", post(handle_client_request))
        .with_state(shared_state);

    println!("wAI Rust Backend running on port 4000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler function
async fn handle_client_request(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> String {
    let channel = state.amqp_conn.create_channel().await.unwrap();
    let payload_bytes = serde_json::to_vec(&payload).unwrap();

    // Push to Queue
    channel.basic_publish(
        "", "wai_prompts",
        BasicPublishOptions::default(),
        &payload_bytes,
        BasicProperties::default(),
    ).await.unwrap();

    format!("Prompt enqueued for client: {}", payload.client_id)
}