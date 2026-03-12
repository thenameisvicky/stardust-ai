use axum::{
    routing::{get, post},
    Json, Router,
};
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties};
use prometheus::{histogram_opts, opts, Counter, Encoder, Histogram, Registry, TextEncoder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct GenerateRequest {
    prompt: String,
}

#[derive(Serialize, Deserialize)]
struct GenerateResponse {
    message: String,
}

struct AppState {
    channel: lapin::Channel,
    api_requests: Counter,
    api_failures: Counter,
    api_latency: Histogram,
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

    // Prometheus
    let registry = Registry::new();

    let api_requests =
        Counter::with_opts(opts!("api_requests_total", "Total API requests")).unwrap();

    let api_failures =
        Counter::with_opts(opts!("api_failures_total", "Total failed API requests")).unwrap();

    let api_latency = Histogram::with_opts(histogram_opts!(
        "api_latency_ms",
        "API latency",
        vec![10.0, 50.0, 100.0, 200.0, 500.0, 1000.0]
    ))
    .unwrap();

    registry.register(Box::new(api_requests.clone())).unwrap();
    registry.register(Box::new(api_failures.clone())).unwrap();
    registry.register(Box::new(api_latency.clone())).unwrap();

    let state = Arc::new(AppState {
        channel,
        api_requests,
        api_failures,
        api_latency,
    });

    let app = Router::new()
        .route("/api/v1/generate", post(generate))
        .route("/metrics", get(metrics))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server running on 3000");

    axum::serve(listener, app).await.unwrap();
}

async fn generate(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(payload): Json<GenerateRequest>,
) -> Json<GenerateResponse> {
    state.api_requests.inc();

    let timer = state.api_latency.start_timer();

    let message = serde_json::to_vec(&payload).unwrap();

    match state
        .channel
        .basic_publish(
            "",
            "LLM_INFERENCE",
            BasicPublishOptions::default(),
            &message,
            BasicProperties::default(),
        )
        .await
    {
        Ok(_) => {
            timer.observe_duration();

            Json(GenerateResponse {
                message: "Request enqueued".to_string(),
            })
        }

        Err(_) => {
            state.api_failures.inc();
            timer.observe_duration();

            Json(GenerateResponse {
                message: "Failed".to_string(),
            })
        }
    }
}

async fn metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}
