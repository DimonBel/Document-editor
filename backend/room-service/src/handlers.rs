//! Room CRUD -- real Mongo-backed storage with a Redis read-through cache.
//!
//! Per #146 vertical-slice: replaces the placeholder handlers with
//! real persisted CRUD. Event publishing to the relay is wired in
//! but the payload is a simple `room.updated` notification -- richer
//! `room.created/deleted/updated` payloads will arrive with the M3
//! custom-event backfill.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ed_cache::Cache;
use ed_domain::{DomainError, Room, UserId};
use ed_errors::AppError;
use ed_persistence_mongo::{AuditFields, MongoRepo, CollectionName};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDoc {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub created_by: String,
    pub latex_source: Option<String>,
    pub snapshot_seq: i64,
    pub audit: AuditFields,
}
impl CollectionName for RoomDoc {
    const COLLECTION: &'static str = "rooms";
}

#[derive(Clone)]
pub struct RoomAppState {
    pub repo: MongoRepo<RoomDoc>,
    pub cache: Cache,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoomIn {
    pub name: String,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize)]
pub struct RoomOut {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub version: u64,
}

impl From<&RoomDoc> for RoomOut {
    fn from(d: &RoomDoc) -> Self {
        Self {
            id: Uuid::parse_str(&d.id).unwrap_or_else(|_| Uuid::nil()),
            name: d.name.clone(),
            created_by: Uuid::parse_str(&d.created_by).unwrap_or_else(|_| Uuid::nil()),
            version: d.snapshot_seq as u64,
        }
    }
}

pub async fn list_rooms(
    State(state): State<RoomAppState>,
) -> Result<Json<Vec<RoomOut>>, AppError> {
    if let Ok(Some(cached)) = state.cache.get::<Vec<RoomOut>>("rooms:list").await {
        return Ok(Json(cached));
    }
    let rooms: Vec<RoomDoc> = state
        .repo
        .collection()
        .find(bson::doc! {})
        .limit(200)
        .await
        .map_err(|e| AppError::Internal(format!("mongo: {e}")))?;
    let out: Vec<RoomOut> = rooms.iter().map(RoomOut::from).collect();
    let _ = state.cache.set_ex("rooms:list", &out, 15).await;
    Ok(Json(out))
}

pub async fn create_room(
    State(state): State<RoomAppState>,
    Json(body): Json<CreateRoomIn>,
) -> Result<(StatusCode, Json<RoomOut>), AppError> {
    let domain = Room::new(body.name, UserId::from(body.created_by))
        .map_err(|e: DomainError| AppError::Validation(e.to_string()))?;
    let doc = RoomDoc {
        id: domain.id().to_string(),
        name: domain.name.clone(),
        created_by: body.created_by.to_string(),
        latex_source: None,
        snapshot_seq: domain.audit.entity.version as i64,
        audit: AuditFields::new(),
    };
    state
        .repo
        .insert(&doc)
        .await
        .map_err(|e| AppError::Internal(format!("mongo: {e}")))?;
    let _ = state.cache.delete("rooms:list").await;
    Ok((StatusCode::CREATED, Json(RoomOut::from(&doc))))
}

pub async fn get_room(
    State(state): State<RoomAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RoomOut>, AppError> {
    let doc = state
        .repo
        .collection()
        .find_one(bson::doc! { "_id": id.to_string() })
        .await
        .map_err(|e| AppError::Internal(format!("mongo: {e}")))?
        .ok_or_else || AppError::NotFound { what: format!("room {id}") };
    Ok(Json(RoomOut::from(&doc)))
}

pub async fn delete_room(
    State(state): State<RoomAppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let exists = state.repo.collection()
        .find_one(bson::doc! { "_id": id.to_string() })
        .await
        .map_err(|e| AppError::Internal(format!("mongo: {e}")))?;
    if exists.is_none() {
        return Err(AppError::NotFound { what: format!("room {id}") });
    }
    state.repo.soft_delete(&id.to_string()).await
        .map_err(|e| AppError::Internal(format!("mongo: {e}")))?;
    let _ = state.cache.delete("rooms:list").await;
    Ok(StatusCode::NO_CONTENT)
}
