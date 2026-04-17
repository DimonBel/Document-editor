use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use actix::Recipient;
use chrono::Utc;
use uuid::Uuid;

use crate::broadcast::dispatcher::OutboundMessage;
use crate::crdt::state::DocumentState;
use crate::models::client::ClientInfo;
use crate::models::operation::Operation;
use crate::models::room::RoomInfo;

// ---------------------------------------------------------------------------
// Internal room representation
// ---------------------------------------------------------------------------

struct RoomClient {
    info: ClientInfo,
    addr: Recipient<OutboundMessage>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RoomData {
    id: String,
    name: String,
    latex_source: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

struct Room {
    id: String,
    name: String,
    clients: HashMap<String, RoomClient>,
    document: DocumentState,
    latex_source: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Room {
    fn info(&self) -> RoomInfo {
        RoomInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            client_count: self.clients.len(),
        }
    }

    fn to_data(&self) -> RoomData {
        RoomData {
            id: self.id.clone(),
            name: self.name.clone(),
            latex_source: self.latex_source.clone(),
            created_at: self.created_at,
        }
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

    fn broadcast_to_room_except(&self, sender_id: &str, msg: &str) {
        for (id, client) in &self.clients {
            if id != sender_id {
                client.addr.do_send(OutboundMessage(msg.to_owned()));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public manager
// ---------------------------------------------------------------------------

pub struct RoomManager {
    rooms: HashMap<String, Room>,
    data_path: PathBuf,
}

impl RoomManager {
    pub fn new() -> Self {
        Self::with_data_dir(PathBuf::from("data"))
    }

    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        let data_path = data_dir.join("rooms.json");
        let rooms = if data_path.exists() {
            match fs::read_to_string(&data_path) {
                Ok(content) => match serde_json::from_str::<Vec<RoomData>>(&content) {
                    Ok(room_data_list) => room_data_list
                        .into_iter()
                        .map(|rd| {
                            let room = Room {
                                id: rd.id.clone(),
                                name: rd.name.clone(),
                                clients: HashMap::new(),
                                document: DocumentState::new(),
                                latex_source: rd.latex_source.clone(),
                                created_at: rd.created_at,
                            };
                            (rd.id, room)
                        })
                        .collect(),
                    Err(e) => {
                        log::warn!("Failed to parse rooms.json: {}", e);
                        HashMap::new()
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read rooms.json: {}", e);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        if let Some(parent) = data_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        RoomManager { rooms, data_path }
    }

    fn save_to_file(&self) {
        let rooms_data: Vec<RoomData> = self.rooms.values().map(|r| r.to_data()).collect();

        match serde_json::to_string_pretty(&rooms_data) {
            Ok(json) => {
                if let Err(e) = fs::write(&self.data_path, json) {
                    log::error!("Failed to save rooms: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize rooms: {}", e);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Room lifecycle
    // -----------------------------------------------------------------------

    pub fn create_room(&mut self, name: String) -> RoomInfo {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let room = Room {
            id: id.clone(),
            name: name.clone(),
            clients: HashMap::new(),
            document: DocumentState::new(),
            latex_source: None,
            created_at: now,
        };
        self.rooms.insert(id.clone(), room);
        self.save_to_file();
        RoomInfo {
            id,
            name,
            created_at: now,
            client_count: 0,
        }
    }

    pub fn get_room_info(&self, room_id: &str) -> Option<RoomInfo> {
        self.rooms.get(room_id).map(Room::info)
    }

    pub fn list_rooms(&self) -> Vec<RoomInfo> {
        self.rooms.values().map(|r| r.info()).collect()
    }

    // -----------------------------------------------------------------------
    // Client lifecycle
    // -----------------------------------------------------------------------

    /// Register a new client and return the JSON sync payload they should receive.
    /// If the room does not exist, it is created automatically.
    pub fn register_client(
        &mut self,
        room_id: &str,
        client_id: String,
        name: String,
        addr: Recipient<OutboundMessage>,
    ) -> String {
        if !self.rooms.contains_key(room_id) {
            self.create_room(format!("Room {room_id}"));
        }

        let room = self.rooms.get_mut(room_id).expect("just created");

        room.clients.insert(
            client_id.clone(),
            RoomClient {
                info: ClientInfo {
                    id: client_id.clone(),
                    name: name.clone(),
                },
                addr,
            },
        );

        let elements: Vec<_> = room
            .document
            .ordered_elements()
            .into_iter()
            .cloned()
            .collect();
        let clients: Vec<_> = room.clients.values().map(|c| &c.info).cloned().collect();

        serde_json::json!({
            "type": "sync",
            "elements": elements,
            "clients": clients,
            "latexSource": room.latex_source,
        })
        .to_string()
    }

    pub fn unregister_client(&mut self, room_id: &str, client_id: &str) {
        let Some(room) = self.rooms.get_mut(room_id) else {
            return;
        };
        room.clients.remove(client_id);

        let msg = serde_json::json!({
            "type": "user_left",
            "clientId": client_id,
        })
        .to_string();
        room.broadcast_all(&msg);
    }

    // -----------------------------------------------------------------------
    // Message handling
    // -----------------------------------------------------------------------

    pub fn apply_operation(&mut self, room_id: &str, sender_id: &str, op: Operation) {
        let Some(room) = self.rooms.get_mut(room_id) else {
            return;
        };
        room.document.apply(op.clone());

        let msg = serde_json::json!({
            "type": "operation",
            "operation": op,
            "senderId": sender_id,
        })
        .to_string();
        room.broadcast_except(sender_id, &msg);
    }

    pub fn broadcast_cursor(&self, room_id: &str, sender_id: &str, cursor_msg: serde_json::Value) {
        let Some(room) = self.rooms.get(room_id) else {
            return;
        };
        room.broadcast_except(sender_id, &cursor_msg.to_string());
    }

    pub fn announce_join(&self, room_id: &str, joiner_id: &str, joiner_name: &str) {
        let Some(room) = self.rooms.get(room_id) else {
            return;
        };
        let msg = serde_json::json!({
            "type": "user_joined",
            "client": { "id": joiner_id, "name": joiner_name },
        })
        .to_string();
        room.broadcast_except(joiner_id, &msg);
    }

    pub fn broadcast_to_room_except(&self, room_id: &str, sender_id: &str, msg: serde_json::Value) {
        let Some(room) = self.rooms.get(room_id) else {
            return;
        };
        room.broadcast_to_room_except(sender_id, &msg.to_string());
    }

    pub fn update_latex_source(&mut self, room_id: &str, sender_id: &str, source: String) {
        {
            let Some(room) = self.rooms.get_mut(room_id) else {
                return;
            };
            room.latex_source = Some(source.clone());
        }
        self.save_to_file();

        let msg = serde_json::json!({
            "type": "latex_source",
            "senderId": sender_id,
            "source": source,
        })
        .to_string();

        if let Some(room) = self.rooms.get(room_id) {
            room.broadcast_except(sender_id, &msg);
        }
    }

    pub fn get_latex_source(&self, room_id: &str) -> Option<String> {
        self.rooms.get(room_id).and_then(|r| r.latex_source.clone())
    }
}
