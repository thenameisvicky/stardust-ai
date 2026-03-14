use axum::{routing::post, Router};
use std::sync::Arc;

use super::handler::chat_handler;
use crate::state::AppState;

pub fn agent_routes() -> Router<Arc<AppState>> {
    Router::new().route("/chat", post(chat_handler))
}
