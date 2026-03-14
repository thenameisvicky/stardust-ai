use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.ws_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        socket.send(Message::Text(msg.into())).await.unwrap();
    }
}
