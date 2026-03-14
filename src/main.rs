use std::sync::Arc;

use lapin::{Connection, ConnectionProperties};
use prometheus::{Counter, Registry};
use reqwest::Client;
use std::collections::HashMap;
use tokio::sync::Mutex;

mod api;
mod core;
mod modules;
mod state;

use state::{AppState, Config};

#[tokio::main]
async fn main() {
    let conn = Connection::connect("amqp://127.0.0.1:5672/%2f", ConnectionProperties::default())
        .await
        .unwrap();

    let registry = Registry::new();

    let api_requests = Counter::new("api_requests_total", "Total API").unwrap();

    registry.register(Box::new(api_requests.clone())).unwrap();

    let state = Arc::new(AppState {
        amqp: Arc::new(conn),
        http_client: Client::new(),
        config: Config {
            ollama_url: "http://localhost:11434".to_string(),
        },
        api_requests,
        prom_registry: registry,
        clients: Arc::new(Mutex::new(HashMap::new())),
    });

    tokio::spawn(core::queue::consumer::run(state.clone()));

    api::router::run(state).await;
}
