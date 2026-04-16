use std::collections::HashMap;

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
    /// Type-erased actor address; avoids a circular dependency with the WS handler.
    addr: Recipient<OutboundMessage>,
}

struct Room {
    id: String,
    name: String,
    clients: HashMap<String, RoomClient>,
    document: DocumentState,
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
}

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: HashMap::new(),
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
            created_at: now,
        };
        self.rooms.insert(id.clone(), room);
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
        self.rooms.values().map(Room::info).collect()
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
}
