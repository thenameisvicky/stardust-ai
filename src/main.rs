use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use lapin::{Connection, ConnectionProperties};
use prometheus::{Counter, Registry};
use qdrant_client::Qdrant;
use reqwest::Client;

mod api;
mod core;
mod modules;
mod state;

use crate::modules::ingestion::store::create_collection;
use state::{AppState, Config};

#[tokio::main]
async fn main() {
    let conn = Connection::connect("amqp://127.0.0.1:5672/%2f", ConnectionProperties::default())
        .await
        .unwrap();

    let registry = Registry::new();

    let api_requests = Counter::new("api_requests_total", "Total API").unwrap();

    let qdrant_client = Qdrant::from_url("http://localhost:6334")
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    registry.register(Box::new(api_requests.clone())).unwrap();

    let state = Arc::new(AppState {
        amqp: Arc::new(conn),
        http_client: Client::new(),
        config: Config {
            ollama_url: "http://localhost:11434".to_string(),
        },
        api_requests,
        prom_registry: registry,
        clients: DashMap::new(),
        qdrant_client,
    });

    create_collection(&state.qdrant_client).await;

    for _ in 0..4 {
        tokio::spawn(core::queue::consumer::run(state.clone()));
    }

    api::router::run(state).await;
}
