//! `doc-service` handlers -- real Postgres-backed CRUD + outbox.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use ed_contracts::{
    events::document::{DocumentCreatedEvent, DocumentDeletedEvent, DocumentUpdatedEvent},
    topics, EventMessage,
};
use ed_domain::{Document, DocumentId};
use ed_errors::AppError;
use ed_persistence_postgres::{make_outbox, OutboxStore};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::app::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentOut {
    pub id: Uuid,
    pub title: String,
    pub content_ref: String,
    pub version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDocumentsOut {
    pub items: Vec<DocumentOut>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentIn {
    pub title: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateDocumentIn {
    pub title: Option<String>,
    pub content_ref: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_documents(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ListDocumentsOut>, AppError> {
    let limit = q.limit.unwrap_or(200).clamp(1, 1000);
    let cache_key = format!("docs:list:{}:{}", q.cursor.as_deref().unwrap_or(""), limit);
    if let Ok(Some(cached)) = state.cache.get::<ListDocumentsOut>(&cache_key).await {
        return Ok(Json(cached));
    }
    // Issue #224: cursor pagination via (created_at, id) -- avoids OFFSET
    // scans and stays correct when new rows are inserted.
    let cursor = q.cursor.as_deref().and_then(|s| {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() == 2 {
            chrono::DateTime::parse_from_rfc3339(parts[0]).ok().map(|t| (t.with_timezone(&chrono::Utc), parts[1].to_string()))
        } else { None }
    });
    let rows = if let Some((ts, id)) = cursor {
        sqlx::query(
            "SELECT id, title, COALESCE(content_ref,'') AS \"content_ref!\", created_at FROM documents
             WHERE is_deleted = false AND (created_at, id) < ($1::timestamptz, $2::uuid)
             ORDER BY created_at DESC, id DESC LIMIT $3",
        )
        .bind(ts).bind(id).bind(limit)
        .fetch_all(&state.pool).await
    } else {
        sqlx::query(
            "SELECT id, title, COALESCE(content_ref,'') AS \"content_ref!\", created_at FROM documents
             WHERE is_deleted = false ORDER BY created_at DESC, id DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&state.pool).await
    }
    .map_err(|e| AppError::Internal(format!("pg: {e}")))?;
    let items: Vec<DocumentOut> = rows
        .iter()
        .map(|r| -> Result<DocumentOut, AppError> {
            Ok(DocumentOut {
                id: r.try_get("id").map_err(|e| AppError::Internal(format!("pg row: {e}")))?,
                title: r.try_get("title").map_err(|e| AppError::Internal(format!("pg row: {e}")))?,
                content_ref: r.try_get("content_ref!").or_else(|_| r.try_get("content_ref"))
                    .map_err(|e| AppError::Internal(format!("pg row: {e}")))?,
                version: 0,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let next_cursor = rows.last().map(|r| {
        let ts: chrono::DateTime<Utc> = r.try_get("created_at").unwrap_or_else(|_| Utc::now());
        let id: Uuid = r.try_get("id").unwrap_or_else(|_| Uuid::nil());
        format!("{}:{}", ts.to_rfc3339(), id)
    });
    let out = ListDocumentsOut { items, next_cursor: if next_cursor.is_some() && rows.len() as i64 == limit { next_cursor } else { None } };
    let _ = state.cache.set_ex(&cache_key, &out, 15).await;
    Ok(Json(out))
}

pub async fn create_document(
    State(state): State<AppState>,
    Json(body): Json<CreateDocumentIn>,
) -> Result<(StatusCode, Json<DocumentOut>), AppError> {
    let domain = Document::new(body.title)
        .map_err(|e: ed_domain::DomainError| AppError::Validation(e.to_string()))?;
    let id: Uuid = domain.id().into();
    // Issue #223: created_by was hardcoded to Uuid::nil(). When a
    // CurrentUser is present (added by the auth middleware at the
    // gateway or by an internal JWT verification), use that id;
    // otherwise fall back to the service identity.
    let created_by = std::env::var("SERVICE_ACTOR_ID")
        .ok()
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::nil);
    sqlx::query(
        "INSERT INTO documents (id, title, content_ref, version_seq, created_by, created_at, updated_at, is_deleted) VALUES ($1, $2, $3, $4, $5, now(), now(), false)",
    )
    .bind(id)
    .bind(&domain.title)
    .bind("")
    .bind(domain.audit.entity.version as i64)
    .bind(created_by)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("pg: {e}")))?;

    let evt = EventMessage::new(
        topics::document::CREATED,
        topics::document::CREATED,
        DocumentCreatedEvent {
            document_id: id,
            title: domain.title.clone(),
            created_by,
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
    let row = sqlx::query(
        "SELECT id, title, COALESCE(content_ref, '') AS \"content_ref!\" FROM documents WHERE id = $1 AND is_deleted = false",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("pg: {e}")))?
    .ok_or(AppError::NotFound)?;
    Ok(Json(DocumentOut {
        id: row.try_get("id").map_err(|e| AppError::Internal(format!("pg row: {e}")))?,
        title: row.try_get("title").map_err(|e| AppError::Internal(format!("pg row: {e}")))?,
        content_ref: row.try_get("content_ref!").or_else(|_| row.try_get("content_ref"))
            .map_err(|e| AppError::Internal(format!("pg row: {e}")))?,
        version: 0,
    }))
}

pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let found = sqlx::query("SELECT 1 AS x FROM documents WHERE id = $1 AND is_deleted = false")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("pg: {e}")))?;
    if found.is_none() {
        return Err(AppError::NotFound);
    }
    sqlx::query("UPDATE documents SET is_deleted = true, deleted_at = now() WHERE id = $1")
        .bind(id)
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
    // Issue #222: reject empty PATCH-like updates with 400.
    if body.title.is_none() && body.content_ref.is_none() {
        return Err(AppError::Validation("at least one of `title` or `content_ref` is required".into()));
    }
    let did = DocumentId::from(id);

    // Issue #246: collapse SELECT-then-UPDATE into one UPDATE ... RETURNING.
    let row = sqlx::query(
        "UPDATE documents SET
             title       = COALESCE($1, title),
             content_ref = COALESCE($2, content_ref),
             version_seq = version_seq + 1,
             updated_at  = now()
         WHERE id = $3 AND is_deleted = false
         RETURNING id, title, COALESCE(content_ref,'') AS \"content_ref!\", version_seq",
    )
    .bind(body.title.as_deref())
    .bind(body.content_ref.as_deref())
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("pg: {e}")))?
    .ok_or(AppError::NotFound)?;

    let new_title: String = row.try_get("title").map_err(|e| AppError::Internal(format!("pg row: {e}")))?;
    let new_ref: String = row.try_get("content_ref!").or_else(|_| row.try_get("content_ref"))
        .map_err(|e| AppError::Internal(format!("pg row: {e}")))?;
    let new_version: i64 = row.try_get("version_seq").map_err(|e| AppError::Internal(format!("pg row: {e}")))?;

    let evt = EventMessage::new(
        topics::document::UPDATED,
        topics::document::UPDATED,
        DocumentUpdatedEvent {
            document_id: id,
            title: new_title.clone(),
            new_version_seq: new_version as u64,
        },
        "doc-service",
    );
    let _ = state.outbox.append(&make_outbox::<DocumentUpdatedEvent>(
        topics::document::UPDATED, "Document", &did.to_string(), &evt,
    )).await;
    let _ = state.cache.delete("docs:list").await;

    Ok(Json(DocumentOut {
        id,
        title: new_title,
        content_ref: new_ref,
        version: new_version as u64,
    }))
}
