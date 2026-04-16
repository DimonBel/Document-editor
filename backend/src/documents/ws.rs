use std::sync::{Arc, Mutex};

use actix::{Actor, ActorContext, AsyncContext, Handler, Running, StreamHandler};
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;
use uuid::Uuid;

use crate::broadcast::dispatcher::OutboundMessage;
use crate::documents::manager::DocumentManager;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum InboundDocMsg {
    JoinDoc {
        #[serde(rename = "docId")]
        doc_id: String,
        #[serde(rename = "clientId")]
        client_id: String,
        name: String,
    },
    DocContentUpdate {
        content: String,
    },
}

pub struct DocWsSession {
    pub id: String,
    pub doc_id: String,
    pub name: String,
    pub docs: web::Data<Arc<Mutex<DocumentManager>>>,
}

impl Actor for DocWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if let Ok(mut mgr) = self.docs.lock() {
            mgr.leave_document(&self.doc_id, &self.id);
        }
        Running::Stop
    }
}

impl Handler<OutboundMessage> for DocWsSession {
    type Result = ();

    fn handle(&mut self, msg: OutboundMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for DocWsSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut ws::WebsocketContext<Self>) {
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

impl DocWsSession {
    fn on_text(&mut self, text: String, ctx: &mut ws::WebsocketContext<Self>) {
        let msg: InboundDocMsg = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Doc WS parse error: {e} — raw: {text}");
                return;
            }
        };

        match msg {
            InboundDocMsg::JoinDoc { doc_id, client_id, name } => {
                self.id = client_id.clone();
                self.doc_id = doc_id.clone();

                let recipient = ctx.address().recipient::<OutboundMessage>();
                let sync_payload = self
                    .docs
                    .lock()
                    .unwrap()
                    .join_document(&doc_id, client_id.clone(), name.clone(), recipient);

                if let Some(payload) = sync_payload {
                    ctx.text(payload);

                    self.docs
                        .lock()
                        .unwrap()
                        .announce_doc_join(&doc_id, &client_id, &name);
                } else {
                    ctx.text(serde_json::json!({
                        "type": "error",
                        "message": "Document not found"
                    }).to_string());
                    ctx.stop();
                }
            }

            InboundDocMsg::DocContentUpdate { content } => {
                self.docs
                    .lock()
                    .unwrap()
                    .update_content(&self.doc_id, &self.id, content);
            }
        }
    }
}

#[get("/ws/doc/{doc_id}")]
pub async fn doc_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
    docs: web::Data<Arc<Mutex<DocumentManager>>>,
) -> Result<HttpResponse, Error> {
    let _doc_id = path.into_inner();
    let session = DocWsSession {
        id: Uuid::new_v4().to_string(),
        doc_id: String::new(),
        name: "Anonymous".to_string(),
        docs,
    };
    actix_web_actors::ws::start(session, &req, stream)
}