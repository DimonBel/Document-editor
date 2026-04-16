use std::collections::HashMap;

use actix::Recipient;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::broadcast::dispatcher::OutboundMessage;
use crate::documents::crdt::{TextDocument, TextOp};
use crate::models::client::ClientInfo;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInfo {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub client_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    pub id: String,
    pub content: String,
    pub version: u64,
}

struct DocumentClient {
    info: ClientInfo,
    addr: Recipient<OutboundMessage>,
}

struct Document {
    id: String,
    title: String,
    clients: HashMap<String, DocumentClient>,
    text_doc: TextDocument,
    created_at: DateTime<Utc>,
}

impl Document {
    fn info(&self) -> DocumentInfo {
        DocumentInfo {
            id: self.id.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            client_count: self.clients.len(),
        }
    }

    fn sync_payload(&self) -> String {
        serde_json::json!({
            "type": "doc_sync",
            "docId": self.id,
            "content": self.text_doc.content(),
            "version": self.text_doc.version(),
            "clients": self.clients.values().map(|c| &c.info).cloned().collect::<Vec<_>>(),
        })
        .to_string()
    }

    fn broadcast_except(&self, sender_id: &str, msg: &str) {
        for (id, client) in &self.clients {
            if id != sender_id {
                client.addr.do_send(OutboundMessage(msg.to_owned()));
            }
        }
    }

    fn broadcast_all(&self, msg: &str) {
        for client in self.clients.values() {
            client.addr.do_send(OutboundMessage(msg.to_owned()));
        }
    }
}

pub struct DocumentManager {
    documents: HashMap<String, Document>,
}

impl DocumentManager {
    pub fn new() -> Self {
        DocumentManager {
            documents: HashMap::new(),
        }
    }

    pub fn create_document(&mut self, title: String) -> DocumentInfo {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let doc = Document {
            id: id.clone(),
            title: title.clone(),
            clients: HashMap::new(),
            text_doc: TextDocument::new(),
            created_at: now,
        };
        self.documents.insert(id.clone(), doc);
        DocumentInfo {
            id,
            title,
            created_at: now,
            client_count: 0,
        }
    }

    pub fn get_document(&self, doc_id: &str) -> Option<DocumentInfo> {
        self.documents.get(doc_id).map(|d| d.info())
    }

    pub fn list_documents(&self) -> Vec<DocumentInfo> {
        self.documents.values().map(|d| d.info()).collect()
    }

    pub fn join_document(
        &mut self,
        doc_id: &str,
        client_id: String,
        name: String,
        addr: Recipient<OutboundMessage>,
    ) -> Option<String> {
        if !self.documents.contains_key(doc_id) {
            return None;
        }

        let doc = self.documents.get_mut(doc_id).unwrap();

        doc.clients.insert(
            client_id.clone(),
            DocumentClient {
                info: ClientInfo {
                    id: client_id.clone(),
                    name: name.clone(),
                },
                addr,
            },
        );

        Some(doc.sync_payload())
    }

    pub fn leave_document(&mut self, doc_id: &str, client_id: &str) {
        let Some(doc) = self.documents.get_mut(doc_id) else {
            return;
        };
        doc.clients.remove(client_id);

        let msg = serde_json::json!({
            "type": "doc_user_left",
            "clientId": client_id,
        })
        .to_string();
        doc.broadcast_all(&msg);
    }

    pub fn apply_text_op(&mut self, doc_id: &str, sender_id: &str, op: TextOp) {
        let Some(doc) = self.documents.get_mut(doc_id) else {
            return;
        };

        doc.text_doc.apply(op.clone());

        let msg = serde_json::json!({
            "type": "doc_operation",
            "op": op,
            "senderId": sender_id,
        })
        .to_string();
        doc.broadcast_except(sender_id, &msg);
    }

    pub fn announce_doc_join(&self, doc_id: &str, joiner_id: &str, joiner_name: &str) {
        let Some(doc) = self.documents.get(doc_id) else {
            return;
        };
        let msg = serde_json::json!({
            "type": "doc_user_joined",
            "client": { "id": joiner_id, "name": joiner_name },
        })
        .to_string();
        doc.broadcast_except(joiner_id, &msg);
    }
}
