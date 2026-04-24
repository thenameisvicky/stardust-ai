use dashmap::DashMap;
use lapin::Connection;
use prometheus::{Counter, Registry};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct Config {
    pub ollama_url: String,
}

pub struct AppState {
    pub amqp: Arc<Connection>,
    pub http_client: Client,
    pub config: Config,
    pub prom_registry: Registry,
    pub api_requests: Counter,
    pub clients: DashMap<String, Sender<String>>,
    pub qdrant_client: qdrant_client::Qdrant,
}
