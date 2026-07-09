use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomCreatedEvent { pub room_id: Uuid, pub name: String, pub created_by: Uuid, pub occurred_at: DateTime<Utc> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomUpdatedEvent { pub room_id: Uuid, pub name: String, pub new_version: u64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDeletedEvent { pub room_id: Uuid, pub deleted_by: Uuid }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomUserJoinedEvent { pub room_id: Uuid, pub user_id: Uuid, pub client_id: String, pub joined_at: DateTime<Utc> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomUserLeftEvent { pub room_id: Uuid, pub user_id: Uuid, pub client_id: String, pub left_at: DateTime<Utc> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSnapshotRequestedEvent { pub room_id: Uuid, pub from_seq: u64, pub requested_by: Uuid }
