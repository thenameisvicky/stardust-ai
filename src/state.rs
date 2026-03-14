use lapin::Connection;
use prometheus::{Counter, Registry};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Config {
    pub ollama_url: String,
}

pub struct AppState {
    pub amqp: Arc<Connection>,
    pub ws_tx: broadcast::Sender<String>,
    pub http_client: Client,
    pub config: Config,
    pub prom_registry: Registry,
    pub api_requests: Counter,
}
