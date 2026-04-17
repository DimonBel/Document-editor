use std::sync::{Arc, Mutex};

use actix::{Actor, ActorContext, AsyncContext, Handler, Running, StreamHandler};
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;
use uuid::Uuid;

use crate::broadcast::dispatcher::OutboundMessage;
use crate::models::operation::Operation;
use crate::rooms::manager::RoomManager;

// ---------------------------------------------------------------------------
// Inbound message envelope
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum InboundMsg {
    Join {
        #[serde(rename = "clientId")]
        client_id: String,
        name: String,
    },
    Operation {
        operation: Operation,
    },
    Preview {
        #[serde(rename = "senderId")]
        sender_id: String,
        element: serde_json::Value,
    },
    PreviewClear {
        #[serde(rename = "senderId")]
        sender_id: String,
    },
    Cursor {
        #[serde(rename = "clientId")]
        client_id: String,
        position: serde_json::Value,
        name: Option<String>,
    },
    LatexSource {
        #[serde(rename = "senderId")]
        sender_id: String,
        source: String,
    },
}

// ---------------------------------------------------------------------------
// WebSocket session actor
// ---------------------------------------------------------------------------

pub struct WsSession {
    pub id: String,
    pub room_id: String,
    pub name: String,
    pub rooms: web::Data<Arc<Mutex<RoomManager>>>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if let Ok(mut mgr) = self.rooms.lock() {
            mgr.unregister_client(&self.room_id, &self.id);
        }
        Running::Stop
    }
}

// Receive messages queued by the room manager for this session.
impl Handler<OutboundMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: OutboundMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// Handle raw WebSocket frames from the browser.
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(data)) => ctx.pong(&data),
            Ok(ws::Message::Text(text)) => self.on_text(text.to_string(), ctx),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl WsSession {
    fn on_text(&mut self, text: String, ctx: &mut ws::WebsocketContext<Self>) {
        let msg: InboundMsg = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("WS parse error in room {}: {e} — raw: {text}", self.room_id);
                return;
            }
        };

        match msg {
            InboundMsg::Join { client_id, name } => {
                self.id = client_id.clone();
                self.name = name.clone();

                let recipient = ctx.address().recipient::<OutboundMessage>();
                let sync_payload = self
                    .rooms
                    .lock()
                    .unwrap()
                    .register_client(&self.room_id, client_id.clone(), name.clone(), recipient);

                ctx.text(sync_payload);

                self.rooms
                    .lock()
                    .unwrap()
                    .announce_join(&self.room_id, &client_id, &name);
            }

            InboundMsg::Operation { operation } => {
                let room_id = self.room_id.clone();
                let sender_id = self.id.clone();
                self.rooms
                    .lock()
                    .unwrap()
                    .apply_operation(&room_id, &sender_id, operation);
            }

            InboundMsg::Preview { sender_id, element } => {
                let room_id = self.room_id.clone();
                let preview_msg = serde_json::json!({
                    "type": "preview",
                    "senderId": sender_id,
                    "element": element,
                });
                self.rooms
                    .lock()
                    .unwrap()
                    .broadcast_to_room_except(&room_id, &sender_id, preview_msg);
            }

            InboundMsg::PreviewClear { sender_id } => {
                let room_id = self.room_id.clone();
                let preview_msg = serde_json::json!({
                    "type": "preview_clear",
                    "senderId": sender_id,
                });
                self.rooms
                    .lock()
                    .unwrap()
                    .broadcast_to_room_except(&room_id, &sender_id, preview_msg);
            }

            InboundMsg::Cursor {
                client_id,
                position,
                name,
            } => {
                let cursor_msg = serde_json::json!({
                    "type": "cursor",
                    "clientId": client_id,
                    "position": position,
                    "name": name,
                });
                let room_id = self.room_id.clone();
                let sender_id = self.id.clone();
                self.rooms
                    .lock()
                    .unwrap()
                    .broadcast_cursor(&room_id, &sender_id, cursor_msg);
            }

            InboundMsg::LatexSource { sender_id, source } => {
                let room_id = self.room_id.clone();
                let sender_id_clone = self.id.clone();
                self.rooms
                    .lock()
                    .unwrap()
                    .update_latex_source(&room_id, &sender_id_clone, source);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Route handler
// ---------------------------------------------------------------------------

#[get("/ws/{room_id}")]
pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
    rooms: web::Data<Arc<Mutex<RoomManager>>>,
) -> Result<HttpResponse, Error> {
    let room_id = path.into_inner();
    let session = WsSession {
        id: Uuid::new_v4().to_string(),
        room_id,
        name: "Anonymous".to_string(),
        rooms,
    };
    actix_web_actors::ws::start(session, &req, stream)
}
