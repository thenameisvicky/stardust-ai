use axum::{routing::get, Router};
use std::sync::Arc;

use super::handler::download_compose;
use crate::state::AppState;

pub fn miscs_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/download", get(download_compose))
}
