use ed_domain::{DomainError, DomainResult, Room, RoomId, UserId};
use ed_persistence_postgres::{make_outbox, OutboxStore};
use ed_contracts::{EventMessage, topics::room as T};
use ed_contracts::events::room::{RoomCreatedEvent, RoomUpdatedEvent, RoomDeletedEvent};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

pub struct RoomService {
    pub outbox: Arc<dyn OutboxStore>,
}

impl RoomService {
    pub async fn create(&self, name: String, created_by: UserId) -> DomainResult<Room> {
        let room = Room::new(name, created_by)?;
        let evt = EventMessage::new(T::CREATED, T::CREATED, RoomCreatedEvent {
            room_id: room.id().into(), name: room.name.clone(), created_by: created_by.into(), occurred_at: Utc::now(),
        }, "room-service");
        self.outbox.append(&make_outbox(T::CREATED, "Room", &room.id().to_string(), &evt))
            .await.map_err(|e| DomainError::Invariant(format!("outbox: {e}")))?;
        Ok(room)
    }
    pub async fn rename(&self, id: RoomId, new_name: String) -> DomainResult<()> {
        let evt = EventMessage::new(T::UPDATED, T::UPDATED, RoomUpdatedEvent { room_id: id.into(), name: new_name, new_version: 1 }, "room-service");
        self.outbox.append(&make_outbox(T::UPDATED, "Room", &id.to_string(), &evt))
            .await.map_err(|e| DomainError::Invariant(format!("outbox: {e}")))?;
        Ok(())
    }
    pub async fn delete(&self, id: RoomId, by: UserId) -> DomainResult<()> {
        let evt = EventMessage::new(T::DELETED, T::DELETED, RoomDeletedEvent { room_id: id.into(), deleted_by: by.into() }, "room-service");
        self.outbox.append(&make_outbox(T::DELETED, "Room", &id.to_string(), &evt))
            .await.map_err(|e| DomainError::Invariant(format!("outbox: {e}")))?;
        Ok(())
    }
}
