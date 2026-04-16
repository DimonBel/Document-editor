use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use actix::Recipient;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::broadcast::dispatcher::OutboundMessage;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentData {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

struct DocumentClient {
    info: ClientInfo,
    addr: Recipient<OutboundMessage>,
}

struct Document {
    id: String,
    title: String,
    content: String,
    clients: HashMap<String, DocumentClient>,
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
            "content": self.content,
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
    data_path: PathBuf,
}

impl DocumentManager {
    pub fn new(data_dir: PathBuf) -> Self {
        let data_path = data_dir.join("documents.json");
        let documents = if data_path.exists() {
            match fs::read_to_string(&data_path) {
                Ok(content) => match serde_json::from_str::<Vec<DocumentData>>(&content) {
                    Ok(docs) => docs
                        .into_iter()
                        .map(|d| {
                            let doc = Document {
                                id: d.id,
                                title: d.title,
                                content: d.content,
                                clients: HashMap::new(),
                                created_at: d.created_at,
                            };
                            (doc.id.clone(), doc)
                        })
                        .collect(),
                    Err(e) => {
                        log::warn!("Failed to parse documents.json: {}", e);
                        HashMap::new()
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read documents.json: {}", e);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        if let Some(parent) = data_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        DocumentManager {
            documents,
            data_path,
        }
    }

    fn save_to_file(&self) {
        let docs: Vec<DocumentData> = self
            .documents
            .values()
            .map(|d| DocumentData {
                id: d.id.clone(),
                title: d.title.clone(),
                content: d.content.clone(),
                created_at: d.created_at,
            })
            .collect();

        match serde_json::to_string_pretty(&docs) {
            Ok(json) => {
                if let Err(e) = fs::write(&self.data_path, json) {
                    log::error!("Failed to save documents: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize documents: {}", e);
            }
        }
    }

    pub fn create_document(&mut self, title: String) -> DocumentInfo {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let doc = Document {
            id: id.clone(),
            title: title.clone(),
            content: String::new(),
            clients: HashMap::new(),
            created_at: now,
        };
        self.documents.insert(id.clone(), doc);
        self.save_to_file();
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

    pub fn get_content(&self, doc_id: &str) -> Option<String> {
        self.documents.get(doc_id).map(|d| d.content.clone())
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

    pub fn update_content(&mut self, doc_id: &str, sender_id: &str, content: String) {
        let (clients_addr, new_content) = {
            let Some(doc) = self.documents.get_mut(doc_id) else {
                return;
            };

            doc.content = content.clone();
            let clients: Vec<_> = doc
                .clients
                .iter()
                .filter(|(id, _)| *id != sender_id)
                .map(|(_, c)| c.addr.clone())
                .collect();
            (clients, content.clone())
        };

        self.save_to_file();

        let msg = serde_json::json!({
            "type": "doc_content_update",
            "content": new_content,
            "senderId": sender_id,
        })
        .to_string();

        for addr in clients_addr {
            let _ = addr.do_send(OutboundMessage(msg.clone()));
        }
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
