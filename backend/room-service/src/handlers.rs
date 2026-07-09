use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use axum::response::IntoResponse;
use axum::extract::State;
use crate::crdt::{DocumentState, Operation};
use uuid::Uuid;
pub async fn ws_handler(State(_s): State<()>, ws: WebSocketUpgrade) -> impl IntoResponse { ws.on_upgrade(handle) }
async fn handle(mut socket: WebSocket) {
    let mut state = DocumentState::default();
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(t) => {
                if let Ok(op) = serde_json::from_str::<Operation>(&t) { state.apply(op); }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    let _ = socket.send(Message::Close(None)).await;
    let _ = Uuid::new_v4();
}
