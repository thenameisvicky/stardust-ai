use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let client_id = Uuid::new_v4().to_string();

    let (tx, mut rx) = mpsc::channel::<String>(100);

    state.clients.lock().await.insert(client_id.clone(), tx);

    let (mut sender, mut receiver) = socket.split();

    sender
        .send(Message::Text(format!("CLIENT_ID:{}", client_id)))
        .await
        .ok();

    println!("Client connected {}", client_id);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(_) = receiver.next().await {}

    state.clients.lock().await.remove(&client_id);

    println!("Client disconnected {}", client_id);
}
