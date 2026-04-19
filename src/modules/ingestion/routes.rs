use axum::{routing::post, Router};
use std::sync::Arc;

use super::handler::ingest_handler;
use crate::state::AppState;

pub fn ingestion_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/ingest", post(ingest_handler))
}
