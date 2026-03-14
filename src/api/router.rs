use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::core::websocket::handler::ws_handler;
use crate::modules::agent::routes::agent_routes;
use crate::state::AppState;

pub async fn run(state: Arc<AppState>) {
    let static_files_service = ServeDir::new("src/views");

    let app = Router::new()
        .merge(agent_routes())
        .route("/ws", get(ws_handler))
        .nest_service("/", static_files_service)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on 3000");

    axum::serve(listener, app).await.unwrap();
}
