//! `doc-service` handlers -- real Postgres-backed CRUD + outbox.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use ed_cache::Cache;
use ed_contracts::{
    events::document::{DocumentCreatedEvent, DocumentDeletedEvent, DocumentUpdatedEvent},
    topics, EventMessage,
};
use ed_domain::{Document, DocumentId};
use ed_errors::AppError;
use ed_persistence_postgres::{make_outbox, OutboxStore};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::app::AppState;

#[derive(Debug, Serialize)]
pub struct DocumentOut {
    pub id: Uuid,
    pub title: String,
    pub content_ref: String,
    pub version: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentIn {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentIn {
    pub title: Option<String>,
    pub content_ref: Option<String>,
}

pub async fn list_documents(
    State(state): State<AppState>,
) -> Result<Json<Vec<DocumentOut>>, AppError> {
    if let Ok(Some(cached)) = state.cache.get::<Vec<DocumentOut>>("docs:list").await {
        return Ok(Json(cached));
    }
    let rows = sqlx::query!(
        "SELECT id, title, COALESCE(content_ref, '') AS \"content_ref!\" FROM documents
         WHERE is_deleted = false ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("pg: {e}")))?;
    let out: Vec<DocumentOut> = rows
        .into_iter()
        .map(|r| DocumentOut {
            id: r.id,
            title: r.title,
            content_ref: r.content_ref.unwrap_or_default(),
            version: 0,
        })
        .collect();
    let _ = state.cache.set_ex("docs:list", &out, 15).await;
    Ok(Json(out))
}

pub async fn create_document(
    State(state): State<AppState>,
    Json(body): Json<CreateDocumentIn>,
) -> Result<(StatusCode, Json<DocumentOut>), AppError> {
    let domain = Document::new(body.title)
        .map_err(|e: ed_domain::DomainError| AppError::Validation(e.to_string()))?;
    let id = domain.id().into();
    sqlx::query!(
        "INSERT INTO documents (id, title, content_ref, version_seq, created_at, updated_at, is_deleted) VALUES ($1, $2, $3, $4, now(), now(), false)",
        id,
        domain.title,
        "",
        domain.audit.entity.version as i64,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("pg: {e}")))?;

    let evt = EventMessage::new(
        topics::document::CREATED,
        topics::document::CREATED,
        DocumentCreatedEvent {
            document_id: id,
            title: domain.title.clone(),
            created_by: Uuid::nil(),  // populated by auth layer
            occurred_at: Utc::now(),
        },
        "doc-service",
    );
    let _ = state.outbox.append(&make_outbox::<DocumentCreatedEvent>(
        topics::document::CREATED, "Document", &id.to_string(), &evt,
    )).await;

    let _ = state.cache.delete("docs:list").await;
    Ok((
        StatusCode::CREATED,
        Json(DocumentOut {
            id,
            title: domain.title,
            content_ref: String::new(),
            version: 0,
        }),
    ))
}

pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentOut>, AppError> {
    let row = sqlx::query!(
        "SELECT id, title, COALESCE(content_ref, '') AS \"content_ref!\" FROM documents WHERE id = $1 AND is_deleted = false",
        id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("pg: {e}")))?
    .ok_or(AppError::NotFound { what: format!("document {id}") })?;
    Ok(Json(DocumentOut {
        id: row.id,
        title: row.title,
        content_ref: row.content_ref,
        version: 0,
    }))
}

pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let found = sqlx::query!("SELECT 1 AS x FROM documents WHERE id = $1 AND is_deleted = false", id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("pg: {e}")))?;
    if found.is_none() {
        return Err(AppError::NotFound { what: format!("document {id}") });
    }
    sqlx::query!("UPDATE documents SET is_deleted = true, deleted_at = now() WHERE id = $1", id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("pg: {e}")))?;

    let evt = EventMessage::new(
        topics::document::DELETED,
        topics::document::DELETED,
        DocumentDeletedEvent {
            document_id: id,
            deleted_by: Uuid::nil(),
        },
        "doc-service",
    );
    let _ = state.outbox.append(&make_outbox::<DocumentDeletedEvent>(
        topics::document::DELETED, "Document", &id.to_string(), &evt,
    )).await;

    let _ = state.cache.delete("docs:list").await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateDocumentIn>,
) -> Result<Json<DocumentOut>, AppError> {
    let did = DocumentId::from(id);
    let found = sqlx::query!("SELECT title, COALESCE(content_ref, '') AS \"content_ref!\" FROM documents WHERE id = $1 AND is_deleted = false", id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("pg: {e}")))?
        .ok_or(AppError::NotFound { what: format!("document {id}") })?;

    let mut domain = Document::new(found.title.clone())
        .map_err(|e: ed_domain::DomainError| AppError::Validation(e.to_string()))?;
    if let Some(t) = body.title.clone() {
        domain.set_title(t).map_err(|e: ed_domain::DomainError| AppError::Validation(e.to_string()))?;
    }
    domain.audit.entity.version += 1;

    sqlx::query!(
        "UPDATE documents SET title = $1, content_ref = COALESCE($2, content_ref), version_seq = $3, updated_at = now() WHERE id = $4",
        domain.title,
        body.content_ref.unwrap_or(found.content_ref),
        domain.audit.entity.version as i64,
        id,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("pg: {e}")))?;

    let evt = EventMessage::new(
        topics::document::UPDATED,
        topics::document::UPDATED,
        DocumentUpdatedEvent {
            document_id: id,
            title: domain.title.clone(),
            new_version_seq: domain.audit.entity.version,
        },
        "doc-service",
    );
    let _ = state.outbox.append(&make_outbox::<DocumentUpdatedEvent>(
        topics::document::UPDATED, "Document", &did.to_string(), &evt,
    )).await;
    let _ = state.cache.delete("docs:list").await;

    Ok(Json(DocumentOut {
        id,
        title: domain.title,
        content_ref: found.content_ref,
        version: domain.audit.entity.version,
    }))
}
