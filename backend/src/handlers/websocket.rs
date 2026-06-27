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
//
// Sender identity (clientId / senderId) is captured at Join time and
// stored in the session. Every other message is attributed to the
// server-owned self.id, not to whatever the client put on the wire,
// to prevent spoofing (L-23).

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
        // Ignored by the server; session id is used instead.
        #[serde(rename = "senderId")]
        _sender_id: String,
        element: serde_json::Value,
    },
    PreviewClear {
        #[serde(rename = "senderId")]
        _sender_id: String,
    },
    Cursor {
        // Ignored; session id is authoritative.
        #[serde(rename = "clientId")]
        _client_id: String,
        position: serde_json::Value,
        name: Option<String>,
    },
    LatexSource {
        #[serde(rename = "senderId")]
        _sender_id: String,
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
    /// Set to true only after the Join frame is processed. Until then
    /// any non-Join inbound message is rejected so a misbehaving or
    /// hostile client cannot attribute broadcasts to a placeholder
    /// session id (L-24).
    pub joined: bool,
    pub rooms: web::Data<Arc<Mutex<RoomManager>>>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if self.joined {
            if let Ok(mut mgr) = self.rooms.lock() {
                mgr.unregister_client(&self.room_id, &self.id);
            }
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
    fn reject_if_not_joined(&self, ctx: &mut ws::WebsocketContext<Self>, frame: &str) -> bool {
        if self.joined {
            return false;
        }
        log::warn!(
            "WS in room {}: dropped pre-join frame from un-joined session: {}",
            self.room_id,
            frame
        );
        ctx.text(
            serde_json::json!({
                "type": "error",
                "message": "Must send 'join' before any other frame",
            })
            .to_string(),
        );
        true
    }

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
                self.joined = true;

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
                if self.reject_if_not_joined(ctx, "operation") {
                    return;
                }
                let room_id = self.room_id.clone();
                let sender_id = self.id.clone();
                self.rooms
                    .lock()
                    .unwrap()
                    .apply_operation(&room_id, &sender_id, operation);
            }

            InboundMsg::Preview { element, .. } => {
                if self.reject_if_not_joined(ctx, "preview") {
                    return;
                }
                let room_id = self.room_id.clone();
                let sender_id = self.id.clone();
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

            InboundMsg::PreviewClear { .. } => {
                if self.reject_if_not_joined(ctx, "previewClear") {
                    return;
                }
                let room_id = self.room_id.clone();
                let sender_id = self.id.clone();
                let preview_msg = serde_json::json!({
                    "type": "preview_clear",
                    "senderId": sender_id,
                });
                self.rooms
                    .lock()
                    .unwrap()
                    .broadcast_to_room_except(&room_id, &sender_id, preview_msg);
            }

            InboundMsg::Cursor { position, name, .. } => {
                if self.reject_if_not_joined(ctx, "cursor") {
                    return;
                }
                let sender_id = self.id.clone();
                let display_name = name.unwrap_or_else(|| self.name.clone());
                let cursor_msg = serde_json::json!({
                    "type": "cursor",
                    "clientId": sender_id,
                    "position": position,
                    "name": display_name,
                });
                let room_id = self.room_id.clone();
                self.rooms
                    .lock()
                    .unwrap()
                    .broadcast_cursor(&room_id, &sender_id, cursor_msg);
            }

            InboundMsg::LatexSource { source, .. } => {
                if self.reject_if_not_joined(ctx, "latexSource") {
                    return;
                }
                let room_id = self.room_id.clone();
                let sender_id = self.id.clone();
                self.rooms
                    .lock()
                    .unwrap()
                    .update_latex_source(&room_id, &sender_id, source);
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
        joined: false,
        rooms,
    };
    actix_web_actors::ws::start(session, &req, stream)
}
