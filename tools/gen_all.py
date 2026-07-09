#!/usr/bin/env python3
"""Generate ALL source files for the Document-editor refactor.

Run: python tools/gen_all.py
"""
from __future__ import annotations
from pathlib import Path

ROOT = Path(r"C:\Users\dmitrii.belih\Desktop\MyProject\Document-editor")
PKG = ROOT / "packages"
SVC = ROOT / "backend"
GW  = ROOT / "gateway"
INFRA = ROOT / "infra"
DOCS = ROOT / "docs" / "refactor"

for d in (PKG, SVC, GW, INFRA, DOCS):
    d.mkdir(parents=True, exist_ok=True)


def write(rel: str, content: str) -> None:
    p = ROOT / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content.lstrip("\n"), encoding="utf-8")


# ========================================================================
# M0 -- FOUNDATIONS
# ========================================================================
def gen_01():
    return [
        ("Cargo.toml", """[workspace]
resolver = "2"
members = ["packages/*", "backend/*"]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.81"
license = "MIT"

[workspace.dependencies]
thiserror = "1"; anyhow = "1"; tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "v7", "serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
axum = { version = "0.7", features = ["macros", "ws", "http2"] }
tower = "0.5"; tower-http = { version = "0.6", features = ["trace", "cors", "request-id", "util"] }
hyper = "1"; http = "1"
sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio-rustls", "postgres", "macros", "chrono", "uuid", "json", "migrate"] }
mongodb = "3"
bson = { version = "2", features = ["chrono-0_4", "uuid-1"] }
deadpool-redis = "0.18"
redis = { version = "0.27", default-features = false, features = ["tokio-comp", "aio"] }
lapin = "2.5"
jsonwebtoken = "9"
futures = "0.3"
parking_lot = "0.12"
once_cell = "1"
tokio = { version = "1", features = ["full"] }
proptest = "1"
testcontainers = "0.20"
testcontainers-modules = { version = "0.11", features = ["postgres", "mongo", "rabbitmq", "redis"] }
ed-domain = { path = "packages/domain" }
ed-contracts = { path = "packages/contracts" }
ed-errors = { path = "packages/errors" }
ed-observability = { path = "packages/observability" }
ed-auth = { path = "packages/auth" }
ed-cache = { path = "packages/cache" }
ed-persistence-postgres = { path = "packages/persistence-postgres" }
ed-persistence-mongo = { path = "packages/persistence-mongo" }
ed-messaging-rabbitmq = { path = "packages/messaging-rabbitmq" }

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "symbols"
"""),
        ("rust-toolchain.toml", """[toolchain]
channel = "1.81.0"
components = ["rustfmt", "clippy", "rust-analyzer"]
profile = "minimal"
"""),
    ]


def gen_02():
    return [
        ("packages/domain/Cargo.toml", """[package]
name = "ed-domain"
version.workspace = true
edition.workspace = true
description = "Pure domain types: entities, value objects, IDs, errors. No broker/db deps."

[dependencies]
thiserror = { workspace = true }
serde = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
"""),
        ("packages/domain/src/lib.rs", """//! `ed-domain` -- pure domain types. NO infrastructure dependencies.
pub mod entity; pub mod value_object; pub mod ids; pub mod error; pub mod room; pub mod document;
pub use entity::{Entity, AuditableEntity, IRowStamped, IAggregateRoot};
pub use value_object::ValueObject;
pub use ids::{RoomId, DocumentId, UserId, ClientId};
pub use error::{DomainError, DomainResult};
pub use room::Room;
pub use document::Document;
"""),
        ("packages/domain/src/entity.rs", """use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use uuid::Uuid;

pub trait IAggregateRoot {}
pub trait IRowStamped {
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn created_by(&self) -> Option<&str>;
    fn updated_at(&self) -> Option<DateTime<Utc>>;
    fn updated_by(&self) -> Option<&str>;
    fn is_deleted(&self) -> bool;
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn deleted_by(&self) -> Option<&str>;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity<TId: Clone> { pub id: TId, pub version: u64 }
impl<TId: Clone + PartialEq> PartialEq for Entity<TId> {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditableEntity<TId: Clone> {
    #[serde(flatten)]
    pub entity: Entity<TId>,
    pub created_at: Option<DateTime<Utc>>, pub created_by: Option<String>,
    pub updated_at: Option<DateTime<Utc>>, pub updated_by: Option<String>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>, pub deleted_by: Option<String>,
}
impl<TId: Clone> AuditableEntity<TId> {
    pub fn new(id: TId) -> Self {
        Self { entity: Entity { id, version: 0 }, created_at: None, created_by: None,
               updated_at: None, updated_by: None, is_deleted: false, deleted_at: None, deleted_by: None }
    }
}
impl<TId: Clone> IRowStamped for AuditableEntity<TId> {
    fn created_at(&self) -> Option<DateTime<Utc>> { self.created_at }
    fn created_by(&self) -> Option<&str> { self.created_by.as_deref() }
    fn updated_at(&self) -> Option<DateTime<Utc>> { self.updated_at }
    fn updated_by(&self) -> Option<&str> { self.updated_by.as_deref() }
    fn is_deleted(&self) -> bool { self.is_deleted }
    fn deleted_at(&self) -> Option<DateTime<Utc>> { self.deleted_at }
    fn deleted_by(&self) -> Option<&str> { self.deleted_by.as_deref() }
}
"""),
        ("packages/domain/src/value_object.rs", """use std::hash::Hash;
pub trait ValueObject: Eq + Hash + Clone {
    fn get_equality_components(&self) -> Vec<Box<dyn std::any::Any>>;
}
"""),
        ("packages/domain/src/ids.rs", """use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
macro_rules! id_newtype { ($name:ident) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct $name(pub Uuid);
    impl $name { pub fn new() -> Self { Self(Uuid::new_v4()) } }
    impl Default for $name { fn default() -> Self { Self::new() } }
    impl fmt::Display for $name { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
    impl From<Uuid> for $name { fn from(v: Uuid) -> Self { Self(v) } }
    impl From<$name> for Uuid { fn from(v: $name) -> Self { v.0 } }
}; }
id_newtype!(UserId); id_newtype!(RoomId); id_newtype!(DocumentId);
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub String);
impl ClientId { pub fn new() -> Self { Self(Uuid::new_v4().to_string()) } }
impl Default for ClientId { fn default() -> Self { Self::new() } }
impl fmt::Display for ClientId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
"""),
        ("packages/domain/src/error.rs", """use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum DomainError {
    #[error("validation: {0}")] Validation(String),
    #[error("not found: {entity} #{id}")] NotFound { entity: String, id: String },
    #[error("conflict: {0}")] Conflict(String),
    #[error("unauthorized: {0}")] Unauthorized(String),
    #[error("forbidden: {0}")] Forbidden(String),
    #[error("invariant: {0}")] Invariant(String),
}
pub type DomainResult<T> = std::result::Result<T, DomainError>;
"""),
        ("packages/domain/src/room.rs", """use serde::{Deserialize, Serialize};
use crate::entity::AuditableEntity;
use crate::error::{DomainError, DomainResult};
use crate::ids::{RoomId, UserId};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    #[serde(flatten)] pub audit: AuditableEntity<RoomId>,
    pub name: String, pub created_by: UserId,
    pub latex_source: Option<String>, pub snapshot_seq: u64,
}
impl Room {
    pub fn new(name: String, created_by: UserId) -> DomainResult<Self> {
        let n = name.trim();
        if n.is_empty() { return Err(DomainError::Validation("name empty".into())); }
        if n.len() > 128 { return Err(DomainError::Validation("name too long".into())); }
        Ok(Self { audit: AuditableEntity::new(RoomId::new()), name: n.to_string(), created_by, latex_source: None, snapshot_seq: 0 })
    }
    pub fn id(&self) -> RoomId { self.audit.entity.id }
    pub fn rename(&mut self, new_name: String) -> DomainResult<()> {
        let n = new_name.trim();
        if n.is_empty() { return Err(DomainError::Validation("name empty".into())); }
        self.name = n.to_string(); self.audit.entity.version += 1; Ok(())
    }
    pub fn set_latex_source(&mut self, src: Option<String>) { self.latex_source = src; self.audit.entity.version += 1; }
}
"""),
        ("packages/domain/src/document.rs", """use serde::{Deserialize, Serialize};
use crate::entity::AuditableEntity;
use crate::error::{DomainError, DomainResult};
use crate::ids::DocumentId;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    #[serde(flatten)] pub audit: AuditableEntity<DocumentId>,
    pub title: String, pub content_ref: String, pub version_seq: u64,
}
impl Document {
    pub fn new(title: String) -> DomainResult<Self> {
        let t = title.trim();
        if t.is_empty() { return Err(DomainError::Validation("title empty".into())); }
        Ok(Self { audit: AuditableEntity::new(DocumentId::new()), title: t.to_string(), content_ref: String::new(), version_seq: 0 })
    }
    pub fn id(&self) -> DocumentId { self.audit.entity.id }
    pub fn set_title(&mut self, t: String) -> DomainResult<()> {
        let t = t.trim();
        if t.is_empty() { return Err(DomainError::Validation("title empty".into())); }
        self.title = t.to_string(); self.audit.entity.version += 1; Ok(())
    }
}
"""),
    ]


def gen_03():
    return [
        ("packages/contracts/Cargo.toml", """[package]
name = "ed-contracts"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
"""),
        ("packages/contracts/src/lib.rs", """pub mod event_message; pub mod topics; pub mod events;
pub use event_message::{EventMessage, IEventMessage};
"""),
        ("packages/contracts/src/event_message.rs", """use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub trait IEventMessage<T> {
    fn id(&self) -> Uuid; fn occurred_at(&self) -> DateTime<Utc>;
    fn data(&self) -> Option<&T>; fn service_name(&self) -> &str; fn topic(&self) -> &str;
    fn correlation_id(&self) -> &str; fn schema_version(&self) -> &str; fn event_name(&self) -> &str;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage<T> {
    pub id: Uuid,
    #[serde(rename = "occurredAt")] pub occurred_at: DateTime<Utc>,
    pub service_name: String, pub module_id: String,
    pub event_name: String, pub topic: String,
    pub correlation_id: String, pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<T>,
}
impl<T> EventMessage<T> {
    pub fn new(topic: impl Into<String>, event: impl Into<String>, data: T, svc: impl Into<String>) -> Self {
        Self { id: Uuid::new_v4(), occurred_at: Utc::now(), service_name: svc.into(),
            module_id: String::new(), event_name: event.into(), topic: topic.into(),
            correlation_id: Uuid::new_v4().to_string(), schema_version: "1".into(), data: Some(data) }
    }
    pub fn with_correlation(mut self, c: impl Into<String>) -> Self { self.correlation_id = c.into(); self }
}
impl<T> IEventMessage<T> for EventMessage<T> {
    fn id(&self) -> Uuid { self.id } fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn data(&self) -> Option<&T> { self.data.as_ref() } fn service_name(&self) -> &str { &self.service_name }
    fn topic(&self) -> &str { &self.topic } fn correlation_id(&self) -> &str { &self.correlation_id }
    fn schema_version(&self) -> &str { &self.schema_version } fn event_name(&self) -> &str { &self.event_name }
}
"""),
        ("packages/contracts/src/topics.rs", """pub mod room; pub mod document; pub mod latex; pub mod audit;
pub struct Topics;
impl Topics {
    pub fn for_ctx(c: &str, a: &str, e: &str) -> String { format!("{}.{}.{}", c.to_lowercase(), a.to_lowercase(), e.to_lowercase()) }
}
"""),
        ("packages/contracts/src/topics/room.rs", """pub const CREATED: &str = "room.created";
pub const UPDATED: &str = "room.updated";
pub const DELETED: &str = "room.deleted";
pub const USER_JOINED: &str = "room.user_joined";
pub const USER_LEFT: &str = "room.user_left";
pub const SNAPSHOT_REQUESTED: &str = "room.snapshot_requested";
pub const SNAPSHOT: &str = "room.snapshot";
"""),
        ("packages/contracts/src/topics/document.rs", """pub const CREATED: &str = "document.created";
pub const UPDATED: &str = "document.updated";
pub const DELETED: &str = "document.deleted";
pub const COMMIT_RECORDED: &str = "document.commit_recorded";
"""),
        ("packages/contracts/src/topics/latex.rs", """pub const COMPILE_REQUESTED: &str = "latex.compile_requested";
pub const COMPILE_SUCCEEDED: &str = "latex.compile_succeeded";
pub const COMPILE_FAILED: &str = "latex.compile_failed";
pub const DOCX_GENERATED: &str = "latex.docx_generated";
pub const DOCX_FAILED: &str = "latex.docx_failed";
"""),
        ("packages/contracts/src/topics/audit.rs", """pub fn recorded(ctx: &str) -> String { format!("{}.audit.recorded", ctx) }
pub const DEAD_LETTER: &str = "audit.recorded.dlq";
pub const POISON: &str = "audit.recorded.poison";
"""),
        ("packages/contracts/src/events/mod.rs", """pub mod room; pub mod document; pub mod latex; pub mod audit;
"""),
        ("packages/contracts/src/events/room.rs", """use chrono::{DateTime, Utc};
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
"""),
        ("packages/contracts/src/events/document.rs", """use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreatedEvent { pub document_id: Uuid, pub title: String, pub created_by: Uuid, pub occurred_at: DateTime<Utc> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpdatedEvent { pub document_id: Uuid, pub title: String, pub new_version_seq: u64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDeletedEvent { pub document_id: Uuid, pub deleted_by: Uuid }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCommitRecordedEvent { pub document_id: Uuid, pub commit_hash: String, pub author: Uuid, pub recorded_at: DateTime<Utc> }
"""),
        ("packages/contracts/src/events/latex.rs", """use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexCompileRequestedEvent { pub request_id: Uuid, pub document_id: Option<Uuid>, pub source_hash: String, pub max_source_bytes: usize, pub requested_by: Uuid, pub requested_at: DateTime<Utc> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexCompileSucceededEvent { pub request_id: Uuid, pub document_id: Option<Uuid>, pub pdf_artefact_url: String, pub compile_seconds: f64, pub source_hash: String, pub occurred_at: DateTime<Utc> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexCompileFailedEvent { pub request_id: Uuid, pub document_id: Option<Uuid>, pub error: String, pub source_hash: String, pub occurred_at: DateTime<Utc> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexDocxGeneratedEvent { pub request_id: Uuid, pub document_id: Option<Uuid>, pub docx_artefact_url: String, pub occurred_at: DateTime<Utc> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexDocxFailedEvent { pub request_id: Uuid, pub document_id: Option<Uuid>, pub error: String, pub occurred_at: DateTime<Utc> }
"""),
        ("packages/contracts/src/events/audit.rs", """use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecordedEvent { pub service_context: String, pub entity_type: String, pub entity_id: String, pub action: String, pub actor: String, pub payload: Value }
"""),
        ("packages/contracts/schema/event_message.schema.json", """{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "EventMessage",
  "type": "object",
  "required": ["id","occurredAt","serviceName","eventName","topic","correlationId","schemaVersion"],
  "properties": {
    "id":            { "type": "string", "format": "uuid" },
    "occurredAt":    { "type": "string", "format": "date-time" },
    "serviceName":   { "type": "string" },
    "moduleId":      { "type": "string" },
    "eventName":     { "type": "string" },
    "topic":         { "type": "string" },
    "correlationId": { "type": "string" },
    "schemaVersion": { "type": "string" },
    "data":          {}
  }
}
"""),
    ]


def gen_04():
    return [
        ("packages/errors/Cargo.toml", """[package]
name = "ed-errors"
version.workspace = true
edition.workspace = true

[features]
axum = ["dep:axum", "dep:http"]

[dependencies]
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
ed-domain = { workspace = true }
sqlx = { workspace = true }
mongodb = { workspace = true }
lapin = { workspace = true }
axum = { workspace = true, optional = true }
http = { workspace = true, optional = true }
"""),
        ("packages/errors/src/lib.rs", """pub mod problem;
pub use problem::ProblemDetails;
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    #[error(transparent)] Domain(ed_domain::DomainError),
    #[error("infra: {0}")] Infra(String),
    #[error("broker: {0}")] Broker(String),
    #[error("auth: {0}")] Auth(String),
    #[error("validation: {0}")] Validation(String),
    #[error("not found")] NotFound,
    #[error("internal: {0}")] Internal(String),
}
impl AppError {
    pub fn http_status(&self) -> u16 {
        match self {
            AppError::Domain(d) => match d {
                ed_domain::DomainError::NotFound { .. } => 404,
                ed_domain::DomainError::Validation(_) => 422,
                ed_domain::DomainError::Conflict(_) => 409,
                ed_domain::DomainError::Unauthorized(_) => 401,
                ed_domain::DomainError::Forbidden(_) => 403,
                ed_domain::DomainError::Invariant(_) => 400,
            },
            AppError::NotFound => 404,
            AppError::Validation(_) => 422,
            AppError::Auth(_) => 401,
            AppError::Infra(_) | AppError::Broker(_) => 502,
            AppError::Internal(_) => 500,
        }
    }
    pub fn to_problem(&self, instance: Option<String>) -> ProblemDetails { ProblemDetails::from_app(self, instance) }
}
pub type AppResult<T> = std::result::Result<T, AppError>;
impl From<ed_domain::DomainError> for AppError { fn from(v: ed_domain::DomainError) -> Self { AppError::Domain(v) } }
impl From<sqlx::Error> for AppError { fn from(e: sqlx::Error) -> Self { AppError::Infra(format!("sqlx: {e}")) } }
impl From<mongodb::error::Error> for AppError { fn from(e: mongodb::error::Error) -> Self { AppError::Infra(format!("mongo: {e}")) } }
impl From<lapin::Error> for AppError { fn from(e: lapin::Error) -> Self { AppError::Broker(format!("lapin: {e}")) } }
impl From<serde_json::Error> for AppError { fn from(e: serde_json::Error) -> Self { AppError::Internal(format!("json: {e}")) } }
"""),
        ("packages/errors/src/problem.rs", """use serde::{Deserialize, Serialize};
use crate::AppError;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")] pub kind: String, pub title: String, pub status: u16, pub detail: Option<String>, pub instance: Option<String>,
}
impl ProblemDetails {
    pub fn from_app(err: &AppError, instance: Option<String>) -> Self {
        let status = err.http_status();
        let title = match err {
            AppError::Domain(_) => "Domain error", AppError::Infra(_) => "Infrastructure error",
            AppError::Broker(_) => "Broker error", AppError::Auth(_) => "Unauthorized",
            AppError::Validation(_) => "Validation failed", AppError::NotFound => "Not found",
            AppError::Internal(_) => "Internal server error",
        }.to_string();
        Self { kind: format!("about:blank#{status}"), title, status, detail: Some(err.to_string()), instance }
    }
}
"""),
    ]


def gen_05():
    return [
        ("packages/observability/Cargo.toml", """[package]
name = "ed-observability"
version.workspace = true
edition.workspace = true

[dependencies]
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
uuid = { workspace = true }
http = { workspace = true }
"""),
        ("packages/observability/src/lib.rs", """use std::sync::Once;
use tracing_subscriber::{prelude::*, EnvFilter};
static INIT: Once = Once::new();
pub fn init_tracing(service_name: &str, json: bool) {
    INIT.call_once(|| {
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,ed_*=debug"));
        let fmt_layer = if json { tracing_subscriber::fmt::layer().json().boxed() } else { tracing_subscriber::fmt::layer().boxed() };
        let filter_svc = EnvFilter::new(format!("info,ed_={}", service_name));
        let svc_layer = tracing_subscriber::fmt::layer().with_filter(filter_svc).boxed();
        tracing_subscriber::registry().with(env_filter).with(fmt_layer).with(svc_layer).init();
    });
}
pub mod correlation {
    use http::HeaderName;
    use uuid::Uuid;
    pub const CORRELATION_HEADER: HeaderName = HeaderName::from_static("x-correlation-id");
    pub type CorrelationId = String;
    pub fn new() -> CorrelationId { Uuid::new_v4().to_string() }
    pub fn from_headers<'a>(headers: impl IntoIterator<Item = &'a http::HeaderValue>) -> Option<CorrelationId> {
        headers.into_iter().filter_map(|v| v.to_str().ok()).find(|s| !s.is_empty()).map(|s| s.to_string())
    }
}
"""),
    ]


def gen_06():
    return [
        ("packages/auth/Cargo.toml", """[package]
name = "ed-auth"
version.workspace = true
edition.workspace = true

[dependencies]
ed-domain = { workspace = true }
ed-errors = { workspace = true }
axum = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
jsonwebtoken = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
http = { workspace = true }
"""),
        ("packages/auth/src/lib.rs", """pub mod verifier; pub mod current_user; pub mod scopes; pub mod error; pub mod extractor;
pub use verifier::{JwtVerifier, Claims};
pub use current_user::CurrentUser;
pub use scopes::{Role, Scope};
pub use error::AuthError;
pub use extractor::CurrentUserExtractor;
"""),
        ("packages/auth/src/verifier.rs", """use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, pub iss: String, pub aud: String,
    pub exp: usize, pub iat: usize,
    #[serde(default)] pub roles: Vec<String>,
    #[serde(default)] pub scopes: Vec<String>,
    #[serde(default)] pub correlation_id: Option<String>,
}
pub struct JwtVerifier { pub(crate) decoding_key: DecodingKey, pub(crate) issuer: String, pub(crate) audience: String, pub(crate) algorithm: Algorithm }
impl JwtVerifier {
    pub fn new_from_secret(secret: &[u8], issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self { decoding_key: DecodingKey::from_secret(secret), issuer: issuer.into(), audience: audience.into(), algorithm: Algorithm::HS256 }
    }
    pub fn new_from_rsa_pem(pem: &[u8], issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self { decoding_key: DecodingKey::from_rsa_pem(pem).expect("invalid RSA PEM"), issuer: issuer.into(), audience: audience.into(), algorithm: Algorithm::RS256 }
    }
    pub fn verify(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut v = Validation::new(self.algorithm.clone());
        v.set_audience(&[self.audience.clone()]);
        v.set_issuer(&[self.issuer.clone()]);
        decode::<Claims>(token, &self.decoding_key, &v).map(|d| d.claims)
    }
    pub fn roles_unique(&self, c: &Claims) -> HashSet<String> { c.roles.iter().cloned().collect() }
}
"""),
        ("packages/auth/src/current_user.rs", """use ed_domain::UserId;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: UserId, pub email: Option<String>,
    pub roles: Vec<String>, pub scopes: Vec<String>, pub correlation_id: Option<String>,
}
impl CurrentUser {
    pub fn has_role(&self, r: &str) -> bool { self.roles.iter().any(|x| x == r) }
    pub fn has_scope(&self, s: &str) -> bool { self.scopes.iter().any(|x| x == s) }
}
"""),
        ("packages/auth/src/scopes.rs", """use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role { User, Admin, Service }
impl Role { pub fn as_str(&self) -> &'static str { match self { Role::User => "user", Role::Admin => "admin", Role::Service => "service" } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope(pub String);
impl Scope { pub fn parse(s: &str) -> Result<Self, String> { if s.is_empty() { Err("empty".into()) } else { Ok(Self(s.to_string())) } } }
"""),
        ("packages/auth/src/error.rs", """use thiserror::Error;
#[derive(Debug, Clone, Error)]
pub enum AuthError {
    #[error("invalid token: {0}")] InvalidToken(String),
    #[error("missing scope: {0}")] MissingScope(String),
    #[error("missing role: {0}")] MissingRole(String),
}
"""),
        ("packages/auth/src/extractor.rs", """use axum::{async_trait, extract::FromRequestParts, http::{header::AUTHORIZATION, request::Parts}};
use crate::current_user::CurrentUser;
use crate::error::AuthError;
use crate::verifier::JwtVerifier;
pub struct CurrentUserExtractor(pub CurrentUser);
#[async_trait]
impl<S> FromRequestParts<S> for CurrentUserExtractor
where S: Send + Sync, JwtVerifier: axum::extract::FromRef<S> {
    type Rejection = AuthError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verifier = JwtVerifier::from_ref(state);
        let token = parts.headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| AuthError::InvalidToken("missing bearer".into()))?;
        let claims = verifier.verify(token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken("sub is not a UUID".into()))?;
        Ok(CurrentUserExtractor(CurrentUser {
            id: id.into(), email: None, roles: claims.roles, scopes: claims.scopes, correlation_id: claims.correlation_id,
        }))
    }
}
"""),
    ]


def gen_07():
    return [
        ("packages/cache/Cargo.toml", """[package]
name = "ed-cache"
version.workspace = true
edition.workspace = true

[dependencies]
ed-errors = { workspace = true }
deadpool-redis = { workspace = true }
redis = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
"""),
        ("packages/cache/src/lib.rs", """pub mod cache; pub mod rate_limiter; pub mod session; pub mod error;
pub use cache::Cache;
pub use rate_limiter::{RateLimiter, RateLimitDecision};
pub use session::Session;
pub use error::CacheError;
"""),
        ("packages/cache/src/error.rs", """use thiserror::Error;
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("redis: {0}")] Redis(#[from] redis::RedisError),
    #[error("pool: {0}")] Pool(String),
    #[error("json: {0}")] Json(#[from] serde_json::Error),
}
impl From<deadpool_redis::PoolError> for CacheError { fn from(e: deadpool_redis::PoolError) -> Self { Self::Pool(e.to_string()) } }
"""),
        ("packages/cache/src/cache.rs", """use deadpool_redis::Pool;
use redis::AsyncCommands;
use crate::error::CacheError;
#[derive(Clone)]
pub struct Cache { pub pool: Pool }
impl Cache {
    pub fn new(pool: Pool) -> Self { Self { pool } }
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let mut c = self.pool.get().await?;
        let v: Option<String> = c.get(key).await?;
        Ok(match v { Some(s) => Some(serde_json::from_str(&s)?), None => None })
    }
    pub async fn set_ex<T: serde::Serialize>(&self, key: &str, value: &T, ttl_secs: u64) -> Result<(), CacheError> {
        let mut c = self.pool.get().await?;
        let s = serde_json::to_string(value)?;
        let _: () = c.set_ex(key, s, ttl_secs).await?;
        Ok(())
    }
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut c = self.pool.get().await?;
        let _: () = c.del(key).await?;
        Ok(())
    }
}
"""),
        ("packages/cache/src/rate_limiter.rs", """use redis::AsyncCommands;
use deadpool_redis::Pool;
use crate::error::CacheError;
#[derive(Debug, Clone, Copy)]
pub enum RateLimitDecision { Allow, Deny }
pub struct RateLimiter { pub pool: Pool, pub capacity: u32, pub refill_per_sec: u32 }
impl RateLimiter {
    pub fn new(pool: Pool, capacity: u32, refill_per_sec: u32) -> Self { Self { pool, capacity, refill_per_sec } }
    pub async fn try_acquire(&self, key: &str) -> Result<RateLimitDecision, CacheError> {
        let mut c = self.pool.get().await?;
        let bucket = (chrono::Utc::now().timestamp() as u32) / self.refill_per_sec.max(1);
        let full_key = format!("rl:{key}:{bucket}");
        let count: u32 = c.incr(&full_key, 1u32).await?;
        if count == 1 { let _: () = c.expire(&full_key, 60).await?; }
        if count > self.capacity { Ok(RateLimitDecision::Deny) } else { Ok(RateLimitDecision::Allow) }
    }
}
"""),
        ("packages/cache/src/session.rs", """use crate::Cache;
use crate::error::CacheError;
use uuid::Uuid;
#[derive(Clone)]
pub struct Session { cache: Cache, ttl_secs: u64 }
impl Session {
    pub fn new(cache: Cache, ttl_secs: u64) -> Self { Self { cache, ttl_secs } }
    pub fn new_id() -> String { Uuid::new_v4().to_string() }
    pub async fn put<T: serde::Serialize>(&self, id: &str, value: &T) -> Result<(), CacheError> { self.cache.set_ex(&format!("sess:{id}"), value, self.ttl_secs).await }
    pub async fn get<T: serde::de::DeserializeOwned>(&self, id: &str) -> Result<Option<T>, CacheError> { self.cache.get(&format!("sess:{id}")).await }
    pub async fn drop(&self, id: &str) -> Result<(), CacheError> { self.cache.delete(&format!("sess:{id}")).await }
}
"""),
    ]


def gen_08():
    return [
        ("packages/persistence-postgres/Cargo.toml", """[package]
name = "ed-persistence-postgres"
version.workspace = true
edition.workspace = true

[dependencies]
ed-domain = { workspace = true }
ed-contracts = { workspace = true }
ed-errors = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
"""),
        ("packages/persistence-postgres/src/lib.rs", """pub mod platform_db; pub mod outbox; pub mod row_stamp; pub mod error;
pub use platform_db::PlatformDb;
pub use outbox::{OutboxMessage, OutboxStatus, OutboxStore, EfOutboxStore, make_outbox};
pub use row_stamp::{RowStampInterceptor, connect};
pub use error::PgError;
"""),
        ("packages/persistence-postgres/src/error.rs", """use thiserror::Error;
#[derive(Debug, Error)]
pub enum PgError {
    #[error("sqlx: {0}")] Sqlx(#[from] sqlx::Error),
    #[error("not found")] NotFound,
    #[error("migration: {0}")] Migration(String),
}
"""),
        ("packages/persistence-postgres/src/migrations/0001_outbox.sql", """CREATE TABLE IF NOT EXISTS outbox_messages (
    id              UUID PRIMARY KEY,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    topic           VARCHAR(128) NOT NULL,
    aggregate_type  VARCHAR(128) NOT NULL,
    aggregate_id    VARCHAR(128) NOT NULL,
    correlation_id  VARCHAR(64)  NOT NULL,
    payload         JSONB        NOT NULL,
    status          SMALLINT     NOT NULL DEFAULT 0,
    attempt_count   INTEGER      NOT NULL DEFAULT 0,
    last_error      VARCHAR(2048),
    next_attempt_at TIMESTAMPTZ  NOT NULL DEFAULT now(),
    sent_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_outbox_messages_pending ON outbox_messages (next_attempt_at) WHERE status IN (0, 1);
CREATE INDEX IF NOT EXISTS ix_outbox_messages_claim   ON outbox_messages (id)            WHERE status IN (0, 1);
CREATE INDEX IF NOT EXISTS ix_outbox_messages_sent    ON outbox_messages (sent_at)       WHERE status = 3;
"""),
        ("packages/persistence-postgres/src/platform_db.rs", """use sqlx::PgPool;
use crate::error::PgError;
#[derive(Clone)]
pub struct PlatformDb { pub pool: PgPool }
impl PlatformDb {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn acquire(&self) -> Result<sqlx::PgConnection, PgError> { Ok(self.pool.acquire().await?) }
    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, PgError> { Ok(self.pool.begin().await?) }
    pub fn pool(&self) -> &PgPool { &self.pool }
}
"""),
        ("packages/persistence-postgres/src/outbox.rs", """use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use ed_contracts::EventMessage;
use crate::error::PgError;
use async_trait::async_trait;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutboxStatus { Pending = 0, Retrying = 1, InFlight = 2, Sent = 3, DeadLettered = 4 }
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OutboxMessage {
    pub id: Uuid, pub occurred_at: DateTime<Utc>,
    pub topic: String, pub aggregate_type: String, pub aggregate_id: String,
    pub correlation_id: String, pub payload: serde_json::Value,
    pub status: i16, pub attempt_count: i32,
    pub last_error: Option<String>, pub next_attempt_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>, pub created_at: DateTime<Utc>,
}
#[async_trait]
pub trait OutboxStore: Send + Sync {
    async fn append(&self, msg: &OutboxMessage) -> Result<(), PgError>;
    async fn claim_pending(&self, limit: i64) -> Result<Vec<OutboxMessage>, PgError>;
    async fn mark_sent(&self, id: Uuid) -> Result<(), PgError>;
    async fn mark_failed(&self, id: Uuid, err: &str, backoff_secs: i64) -> Result<(), PgError>;
    async fn mark_dead_lettered(&self, id: Uuid, err: &str) -> Result<(), PgError>;
}
pub struct EfOutboxStore { pub pool: PgPool }
#[async_trait]
impl OutboxStore for EfOutboxStore {
    async fn append(&self, m: &OutboxMessage) -> Result<(), PgError> {
        sqlx::query("INSERT INTO outbox_messages (id, occurred_at, topic, aggregate_type, aggregate_id, correlation_id, payload, status, attempt_count, next_attempt_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
            .bind(m.id).bind(m.occurred_at).bind(&m.topic).bind(&m.aggregate_type).bind(&m.aggregate_id)
            .bind(&m.correlation_id).bind(&m.payload).bind(m.status).bind(m.attempt_count).bind(m.next_attempt_at)
            .execute(&self.pool).await?;
        Ok(())
    }
    async fn claim_pending(&self, limit: i64) -> Result<Vec<OutboxMessage>, PgError> {
        let rows = sqlx::query_as::<_, OutboxMessage>("SELECT id, occurred_at, topic, aggregate_type, aggregate_id, correlation_id, payload, status, attempt_count, last_error, next_attempt_at, sent_at, created_at FROM outbox_messages WHERE status IN (0,1) AND next_attempt_at <= now() ORDER BY next_attempt_at ASC LIMIT $1 FOR UPDATE SKIP LOCKED")
            .bind(limit).fetch_all(&self.pool).await?;
        Ok(rows)
    }
    async fn mark_sent(&self, id: Uuid) -> Result<(), PgError> {
        sqlx::query("UPDATE outbox_messages SET status = 3, sent_at = now() WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }
    async fn mark_failed(&self, id: Uuid, err: &str, backoff_secs: i64) -> Result<(), PgError> {
        sqlx::query("UPDATE outbox_messages SET status = 1, attempt_count = attempt_count + 1, last_error = $2, next_attempt_at = now() + ($3::int * interval '1 second') WHERE id = $1")
            .bind(id).bind(err).bind(backoff_secs as i32).execute(&self.pool).await?;
        Ok(())
    }
    async fn mark_dead_lettered(&self, id: Uuid, err: &str) -> Result<(), PgError> {
        sqlx::query("UPDATE outbox_messages SET status = 4, last_error = $2 WHERE id = $1").bind(id).bind(err).execute(&self.pool).await?;
        Ok(())
    }
}
pub fn make_outbox<T: Serialize>(topic: &str, aggregate_type: &str, aggregate_id: &str, evt: &EventMessage<T>) -> OutboxMessage {
    OutboxMessage {
        id: Uuid::new_v4(), occurred_at: Utc::now(),
        topic: topic.to_string(), aggregate_type: aggregate_type.to_string(), aggregate_id: aggregate_id.to_string(),
        correlation_id: evt.correlation_id().to_string(),
        payload: serde_json::to_value(evt).unwrap_or(serde_json::Value::Null),
        status: OutboxStatus::Pending as i16, attempt_count: 0, last_error: None,
        next_attempt_at: Utc::now(), sent_at: None, created_at: Utc::now(),
    }
}
"""),
        ("packages/persistence-postgres/src/row_stamp.rs", """use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use crate::platform_db::PlatformDb;
pub struct RowStampInterceptor;
impl RowStampInterceptor {
    pub async fn ensure(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("packages/persistence-postgres/src/migrations").run(pool).await?;
        Ok(())
    }
}
pub async fn connect(database_url: &str) -> Result<PlatformDb, sqlx::Error> {
    let pool = PgPoolOptions::new().max_connections(20).acquire_timeout(Duration::from_secs(5)).connect(database_url).await?;
    RowStampInterceptor::ensure(&pool).await.map_err(|e| sqlx::Error::Migrate(Box::new(e)))?;
    Ok(PlatformDb::new(pool))
}
"""),
    ]


def gen_09():
    return [
        ("packages/persistence-mongo/Cargo.toml", """[package]
name = "ed-persistence-mongo"
version.workspace = true
edition.workspace = true

[dependencies]
ed-domain = { workspace = true }
ed-errors = { workspace = true }
mongodb = { workspace = true }
bson = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
chrono = { workspace = true }
"""),
        ("packages/persistence-mongo/src/lib.rs", """pub mod mongo_db; pub mod repo; pub mod conventions; pub mod error;
pub use mongo_db::MongoDb;
pub use repo::MongoRepo;
pub use conventions::{AuditFields, CollectionName, to_bson_dt, from_bson_dt};
pub use error::MongoError;
"""),
        ("packages/persistence-mongo/src/error.rs", """use thiserror::Error;
#[derive(Debug, Error)]
pub enum MongoError {
    #[error("mongodb: {0}")] Mongo(#[from] mongodb::error::Error),
    #[error("bson: {0}")] Bson(#[from] bson::ser::Error),
    #[error("not found")] NotFound,
}
"""),
        ("packages/persistence-mongo/src/mongo_db.rs", """use mongodb::{Client, Database};
use crate::error::MongoError;
#[derive(Clone)]
pub struct MongoDb { pub client: Client, pub db_name: String }
impl MongoDb {
    pub async fn connect(url: &str, db_name: impl Into<String>) -> Result<Self, MongoError> {
        let client = Client::with_uri_str(url).await?;
        Ok(Self { client, db_name: db_name.into() })
    }
    pub fn database(&self) -> Database { self.client.database(&self.db_name) }
}
"""),
        ("packages/persistence-mongo/src/conventions.rs", """use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFields {
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
    pub is_deleted: bool, pub deleted_at: Option<DateTime<Utc>>,
}
impl AuditFields {
    pub fn new() -> Self { Self { created_at: Utc::now(), updated_at: Utc::now(), is_deleted: false, deleted_at: None } }
    pub fn touch(&mut self) { self.updated_at = Utc::now(); }
    pub fn soft_delete(&mut self) { self.is_deleted = true; self.deleted_at = Some(Utc::now()); self.updated_at = Utc::now(); }
}
impl Default for AuditFields { fn default() -> Self { Self::new() } }
pub fn to_bson_dt(dt: DateTime<Utc>) -> bson::DateTime { bson::DateTime::from_chrono(dt) }
pub fn from_bson_dt(dt: bson::DateTime) -> DateTime<Utc> { dt.to_chrono() }
pub trait CollectionName { const COLLECTION: &'static str; }
"""),
        ("packages/persistence-mongo/src/repo.rs", """use mongodb::Collection;
use serde::{de::DeserializeOwned, Serialize};
use bson::doc;
use crate::error::MongoError;
use crate::mongo_db::MongoDb;
pub struct MongoRepo<T: Serialize + DeserializeOwned + Send + Sync + 'static> {
    pub db: MongoDb, _phantom: std::marker::PhantomData<T>,
}
impl<T: Serialize + DeserializeOwned + Send + Sync + crate::conventions::CollectionName + 'static> MongoRepo<T> {
    pub fn new(db: MongoDb) -> Self { Self { db, _phantom: std::marker::PhantomData } }
    pub fn collection(&self) -> Collection<T> { self.db.database().collection::<T>(T::COLLECTION) }
    pub async fn find_one(&self, id: &str) -> Result<Option<T>, MongoError> {
        Ok(self.collection().find_one(doc! { "_id": id }).await?)
    }
    pub async fn insert(&self, doc: &T) -> Result<(), MongoError> { self.collection().insert_one(doc).await?; Ok(()) }
    pub async fn replace(&self, id: &str, doc: &T) -> Result<(), MongoError> { self.collection().replace_one(doc! { "_id": id }, doc).await?; Ok(()) }
    pub async fn soft_delete(&self, id: &str) -> Result<(), MongoError> {
        let now = bson::DateTime::now();
        self.collection().update_one(doc! { "_id": id }, doc! { "$set": { "is_deleted": true, "deleted_at": now, "updated_at": now } }).await?;
        Ok(())
    }
}
"""),
    ]


def gen_10():
    return [
        ("packages/messaging-rabbitmq/Cargo.toml", """[package]
name = "ed-messaging-rabbitmq"
version.workspace = true
edition.workspace = true

[dependencies]
ed-domain = { workspace = true }
ed-contracts = { workspace = true }
ed-errors = { workspace = true }
ed-persistence-postgres = { workspace = true }
lapin = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
"""),
        ("packages/messaging-rabbitmq/src/lib.rs", """pub mod event_bus; pub mod publisher; pub mod consumer; pub mod topology; pub mod outbox_relay; pub mod type_resolver; pub mod error;
pub use event_bus::{IEventBus, RabbitEventBus};
pub use publisher::HubProducer;
pub use consumer::ConsumerHandler;
pub use topology::{Topology, ExchangeSpec, QueueSpec, BindingSpec, TopologyDeclaration};
pub use outbox_relay::OutboxRelayService;
pub use type_resolver::TypeObjectResolver;
pub use error::BrokerError;
"""),
        ("packages/messaging-rabbitmq/src/error.rs", """use thiserror::Error;
#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("lapin: {0}")] Lapin(#[from] lapin::Error),
    #[error("json: {0}")] Json(#[from] serde_json::Error),
    #[error("pg: {0}")] Pg(#[from] ed_persistence_postgres::PgError),
    #[error("not connected")] NotConnected,
    #[error("topology mismatch: {0}")] TopologyMismatch(String),
}
"""),
        ("packages/messaging-rabbitmq/src/topology.rs", """use serde::{Deserialize, Serialize};
use crate::error::BrokerError;
use lapin::{Channel, ExchangeKind, options::*, types::FieldTable};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeSpec { pub name: String, pub kind: String, pub durable: bool }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSpec { pub name: String, pub durable: bool, #[serde(default)] pub auto_delete: bool, #[serde(default)] pub dead_letter_exchange: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingSpec { pub source: String, pub destination: String, pub routing_key: String }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Topology { #[serde(default)] pub exchanges: Vec<ExchangeSpec>, #[serde(default)] pub queues: Vec<QueueSpec>, #[serde(default)] pub bindings: Vec<BindingSpec> }
#[async_trait::async_trait]
pub trait TopologyDeclaration: Send + Sync { async fn declare(&self, ch: &Channel) -> Result<(), BrokerError>; }
#[async_trait::async_trait]
impl TopologyDeclaration for Topology {
    async fn declare(&self, ch: &Channel) -> Result<(), BrokerError> {
        for ex in &self.exchanges {
            let kind = match ex.kind.as_str() { "topic" => ExchangeKind::Topic, "fanout" => ExchangeKind::Fanout, "direct" => ExchangeKind::Direct, _ => ExchangeKind::Headers };
            ch.exchange_declare(&ex.name, kind, ExchangeDeclareOptions { durable: ex.durable, ..Default::default() }, FieldTable::default()).await?;
        }
        for q in &self.queues {
            let mut args = FieldTable::default();
            if let Some(dlx) = &q.dead_letter_exchange { args.insert("x-dead-letter-exchange".into(), lapin::types::AMQPValue::LongString(dlx.clone().into())); }
            ch.queue_declare(&q.name, QueueDeclareOptions { durable: q.durable, auto_delete: q.auto_delete, ..Default::default() }, args).await?;
        }
        for b in &self.bindings {
            ch.queue_bind(&b.destination, &b.source, &b.routing_key, QueueBindOptions::default(), FieldTable::default()).await?;
        }
        Ok(())
    }
}
"""),
        ("packages/messaging-rabbitmq/src/event_bus.rs", """use async_trait::async_trait;
use lapin::{Channel, Connection, ConnectionProperties, options::BasicPublishOptions, BasicProperties};
use std::sync::Arc;
use tokio::sync::Mutex;
use ed_contracts::EventMessage;
use crate::error::BrokerError;
use crate::topology::{Topology, TopologyDeclaration};
#[async_trait]
pub trait IEventBus: Send + Sync {
    async fn publish<T: serde::Serialize + Send + Sync>(&self, topic: &str, evt: &EventMessage<T>) -> Result<(), BrokerError>;
    fn channel(&self) -> Channel;
}
pub struct RabbitEventBus { pub conn: Arc<Mutex<Connection>>, pub channel: Channel, pub topology: Topology }
impl RabbitEventBus {
    pub async fn connect(url: &str, topology: Topology) -> Result<Self, BrokerError> {
        let conn = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        topology.declare(&channel).await?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)), channel, topology })
    }
}
#[async_trait]
impl IEventBus for RabbitEventBus {
    async fn publish<T: serde::Serialize + Send + Sync>(&self, topic: &str, evt: &EventMessage<T>) -> Result<(), BrokerError> {
        let payload = serde_json::to_vec(evt)?;
        let confirm = self.channel.basic_publish("ed.events", topic, BasicPublishOptions { mandatory: true, ..Default::default() }, &payload,
            BasicProperties::default().with_content_type("application/json".into())
                .with_correlation_id(evt.correlation_id().to_string().into())
                .with_message_id(evt.id().to_string().into())
        ).await?.await?;
        if confirm.is_nack() { return Err(BrokerError::NotConnected); }
        Ok(())
    }
    fn channel(&self) -> Channel { self.channel.clone() }
}
"""),
        ("packages/messaging-rabbitmq/src/publisher.rs", """use lapin::{Channel, options::BasicPublishOptions, BasicProperties};
use serde::Serialize;
use ed_contracts::EventMessage;
use crate::error::BrokerError;
pub struct HubProducer { pub channel: Channel }
impl HubProducer {
    pub async fn send<T: Serialize>(&self, exchange: &str, topic: &str, evt: &EventMessage<T>) -> Result<(), BrokerError> {
        let payload = serde_json::to_vec(evt)?;
        self.channel.basic_publish(exchange, topic, BasicPublishOptions { mandatory: true, ..Default::default() }, &payload,
            BasicProperties::default().with_content_type("application/json".into())
                .with_correlation_id(evt.correlation_id().to_string().into())
                .with_message_id(evt.id().to_string().into())).await?.await?;
        Ok(())
    }
}
"""),
        ("packages/messaging-rabbitmq/src/consumer.rs", """use async_trait::async_trait;
use ed_contracts::EventMessage;
use crate::error::BrokerError;
use lapin::{message::Delivery, Channel};
#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    type Event: serde::de::DeserializeOwned + Send + Sync;
    async fn handle(&self, evt: EventMessage<Self::Event>, raw: &Delivery, ch: &Channel) -> Result<(), BrokerError>;
}
"""),
        ("packages/messaging-rabbitmq/src/type_resolver.rs", """use lapin::types::{AMQPValue, ShortString};
use lapin::message::Delivery;
use ed_contracts::EventMessage;
use serde::de::DeserializeOwned;
pub const HEADER_TYPE_NAME: &str = "x-ed-type-name";
pub struct TypeObjectResolver;
impl TypeObjectResolver {
    pub fn decode<T: DeserializeOwned>(d: &Delivery) -> Result<EventMessage<T>, serde_json::Error> {
        serde_json::from_slice(&d.data)
    }
    pub fn get_type_name(d: &Delivery) -> Option<String> {
        d.properties.headers().as_ref()?.inner().get(HEADER_TYPE_NAME).and_then(|v| match v {
            AMQPValue::LongString(s) => Some(s.to_string()),
            AMQPValue::ShortString(s) => Some(s.to_string()),
            _ => None,
        })
    }
}
"""),
        ("packages/messaging-rabbitmq/src/outbox_relay.rs", """use ed_persistence_postgres::OutboxStore;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use crate::event_bus::IEventBus;
use crate::error::BrokerError;
use ed_contracts::EventMessage;
use serde_json::Value;
pub struct OutboxRelayService {
    pub store: Arc<dyn OutboxStore>,
    pub bus: Arc<dyn IEventBus>,
    pub poll_interval: Duration, pub batch_size: i64,
    pub max_attempts: i32, pub backoff_base_ms: i64, pub backoff_max_ms: i64,
}
impl OutboxRelayService {
    pub async fn run(self: Arc<Self>) {
        loop {
            if let Err(e) = self.tick().await { error!(error = %e, "outbox relay tick failed"); }
            tokio::time::sleep(self.poll_interval).await;
        }
    }
    pub async fn tick(&self) -> Result<(), BrokerError> {
        let claimed = self.store.claim_pending(self.batch_size).await?;
        if claimed.is_empty() { return Ok(()); }
        info!(count = claimed.len(), "claimed outbox rows");
        for row in claimed {
            let envelope: Result<EventMessage<Value>, _> = serde_json::from_value(row.payload.clone());
            let evt = match envelope { Ok(e) => e, Err(e) => { self.store.mark_dead_lettered(row.id, &format!("decode: {e}")).await?; continue; } };
            match self.bus.publish(&row.topic, &evt).await {
                Ok(()) => { self.store.mark_sent(row.id).await?; }
                Err(e) => {
                    let next_attempt = row.attempt_count + 1;
                    if next_attempt >= self.max_attempts {
                        self.store.mark_dead_lettered(row.id, &e.to_string()).await?;
                        warn!(id = %row.id, "outbox row dead-lettered after max attempts");
                    } else {
                        let backoff = (self.backoff_base_ms * (1 << next_attempt.min(8))).min(self.backoff_max_ms);
                        self.store.mark_failed(row.id, &e.to_string(), backoff / 1000).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
"""),
    ]


# Stub generators for the rest -- they add structured placeholder files.
def stub(n: int, title: str, body_summary: str, files: dict[str, str]) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    for rel, content in files.items():
        out.append((rel, content))
    return out


def gen_11(): return stub(11, "Compose file", "", {
    "infra/docker-compose.yml": """name: ed
services:
  postgres:
    image: postgres:16-alpine
    environment: { POSTGRES_DB: ed, POSTGRES_USER: ed, POSTGRES_PASSWORD: ed }
    ports: ["5432:5432"]
    volumes: ["pg:/var/lib/postgresql/data"]
    healthcheck: { test: ["CMD", "pg_isready", "-U", "ed"], interval: "5s", retries: 10 }
  mongo:
    image: mongo:7
    ports: ["27017:27017"]
    volumes: ["mongo:/data/db"]
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
    healthcheck: { test: ["CMD", "redis-cli", "ping"], interval: "5s", retries: 10 }
  rabbit:
    image: rabbitmq:3.13-management-alpine
    ports: ["5672:5672", "15672:15672"]
    volumes:
      - ./docker/rabbit/definitions.json:/etc/rabbitmq/definitions.json:ro
      - ./docker/rabbit/rabbitmq.conf:/etc/rabbitmq/rabbitmq.conf:ro
    healthcheck: { test: ["CMD", "rabbitmq-diagnostics", "ping"], interval: "10s", retries: 10 }
  room-service:
    build: { context: ., dockerfile: infra/docker/Dockerfile.rust-service }
    args: { SERVICE: room-service }
    environment:
      DATABASE_URL: postgres://ed:ed@postgres:5432/ed
      MONGO_URL: mongodb://mongo:27017/ed
      REDIS_URL: redis://redis:6379
      RABBITMQ_URL: amqp://guest:guest@rabbit:5672/ed
      SERVICE_NAME: room-service
    depends_on:
      postgres: { condition: service_healthy }
      mongo:    { condition: service_started }
      redis:    { condition: service_healthy }
      rabbit:   { condition: service_healthy }
  doc-service:
    build: { context: ., dockerfile: infra/docker/Dockerfile.rust-service }
    args: { SERVICE: doc-service }
    environment:
      DATABASE_URL: postgres://ed:ed@postgres:5432/ed
      MONGO_URL: mongodb://mongo:27017/ed
      REDIS_URL: redis://redis:6379
      RABBITMQ_URL: amqp://guest:guest@rabbit:5672/ed
      SERVICE_NAME: doc-service
    depends_on: [postgres, redis, rabbit]
  latex-service:
    build: { context: ., dockerfile: infra/docker/Dockerfile.rust-service, args: { SERVICE: latex-service, INSTALL_TEX: "1" } }
    environment:
      DATABASE_URL: postgres://ed:ed@postgres:5432/ed
      MONGO_URL: mongodb://mongo:27017/ed
      REDIS_URL: redis://redis:6379
      RABBITMQ_URL: amqp://guest:guest@rabbit:5672/ed
      SERVICE_NAME: latex-service
      LATEX_ARTEFACTS_DIR: /var/lib/latex
    volumes: ["latex_artefacts:/var/lib/latex"]
    depends_on: [postgres, mongo, redis, rabbit]
  gateway:
    build: { context: ./gateway, dockerfile: Dockerfile }
    environment:
      DATABASE_URL: postgres://ed:ed@postgres:5432/ed
      MONGO_URL: mongodb://mongo:27017/ed
      REDIS_URL: redis://redis:6379
      RABBITMQ_URL: amqp://guest:guest@rabbit:5672/ed
      JWT_ISSUER: ed-gateway
      JWT_AUDIENCE: ed-services
      INTERNAL_SERVICE_TOKEN_SECRET: changeme
    ports: ["8080:8080"]
    depends_on:
      rabbit: { condition: service_healthy }
  frontend:
    build: { context: ./frontend }
    ports: ["5173:80"]
    depends_on: [gateway]
volumes:
  pg: {}; mongo: {}; redis: {}; latex_artefacts: {}
"""
})


def gen_12(): return stub(12, "Dockerfile.rust-service", "", {
    "infra/docker/Dockerfile.rust-service": """# syntax=docker/dockerfile:1.7
ARG SERVICE
ARG INSTALL_TEX=""
FROM rust:1.81-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY packages packages
COPY backend backend
RUN cargo build -p ${SERVICE} --release

FROM debian:bookworm-slim
ARG SERVICE
ARG INSTALL_TEX
ENV HOST=0.0.0.0 PORT=8080 SERVICE=${SERVICE}
RUN apt-get update && apt-get install -y ca-certificates libssl3 \\
    $([ "$INSTALL_TEX" = "1" ] && echo "texlive-latex-base texlive-fonts-recommended texlive-latex-recommended texlive-science texlive-pictures") \\
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/${SERVICE} /app/${SERVICE}
COPY infra/docker/rust-runtime.sh /app/runtime.sh
RUN chmod +x /app/runtime.sh
EXPOSE 8080
CMD ["/app/runtime.sh"]
""",
    "infra/docker/rust-runtime.sh": """#!/bin/sh
set -e
exec /app/${SERVICE}
""",
})


def gen_13(): return stub(13, "Dockerfile.gateway", "", {
    "gateway/Dockerfile": """FROM python:3.12-slim
WORKDIR /app
ENV PYTHONDONTWRITEBYTECODE=1 PYTHONUNBUFFERED=1 PIP_NO_CACHE_DIR=1
COPY pyproject.toml ./
COPY app ./app
RUN pip install --no-cache-dir -e .
RUN useradd -r -u 1000 app && chown -R app /app
USER app
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=3 \\
  CMD python -c "import httpx,sys; sys.exit(0 if httpx.get('http://127.0.0.1:8080/health').status_code==200 else 1)"
CMD ["uvicorn", "gateway.app.main:app", "--host", "0.0.0.0", "--port", "8080"]
""",
})


def gen_14(): return stub(14, "RabbitMQ topology", "", {
    "infra/docker/rabbit/definitions.json": """{
  "vhosts": [{ "name": "/" }],
  "exchanges": [
    { "name": "ed.events",    "type": "topic", "durable": true, "vhost": "/" },
    { "name": "ed.events.dlx","type": "topic", "durable": true, "vhost": "/" }
  ],
  "queues": [
    { "name": "ed.room-service",     "durable": true, "auto_delete": false, "vhost": "/", "arguments": { "x-dead-letter-exchange": "ed.events.dlx" } },
    { "name": "ed.doc-service",      "durable": true, "auto_delete": false, "vhost": "/", "arguments": { "x-dead-letter-exchange": "ed.events.dlx" } },
    { "name": "ed.latex-service",    "durable": true, "auto_delete": false, "vhost": "/", "arguments": { "x-dead-letter-exchange": "ed.events.dlx" } },
    { "name": "ed.audit",            "durable": true, "auto_delete": false, "vhost": "/", "arguments": { "x-dead-letter-exchange": "ed.events.dlx" } },
    { "name": "ed.realtime-gateway", "durable": false,"auto_delete": true,  "vhost": "/", "arguments": { "x-dead-letter-exchange": "ed.events.dlx" } }
  ],
  "bindings": [
    { "source": "ed.events", "destination": "ed.room-service",     "destination_type": "queue", "routing_key": "room.*",     "vhost": "/" },
    { "source": "ed.events", "destination": "ed.doc-service",      "destination_type": "queue", "routing_key": "document.*", "vhost": "/" },
    { "source": "ed.events", "destination": "ed.latex-service",    "destination_type": "queue", "routing_key": "latex.*",    "vhost": "/" },
    { "source": "ed.events", "destination": "ed.audit",            "destination_type": "queue", "routing_key": "#.audit.recorded", "vhost": "/" },
    { "source": "ed.events", "destination": "ed.realtime-gateway", "destination_type": "queue", "routing_key": "*",         "vhost": "/" }
  ]
}
""",
    "infra/docker/rabbit/rabbitmq.conf": """management.tcp.port = 15672
default_user_tags.administrator = true
loopback_users.guest = false
""",
})


def gen_15(): return stub(15, "Healthchecks", "", {
    "docs/refactor/M1-notes.md": """# M1 Notes
Healthchecks defined in `infra/docker-compose.yml`:
- postgres: `pg_isready -U ed`
- mongo:    `db.runCommand({ping:1})`
- redis:    `redis-cli ping`
- rabbit:   `rabbitmq-diagnostics ping`
- per-service `/healthz` (added in M2/M3+)

`depends_on: condition: service_healthy` is used for ordering.
""",
})


def gen_16(): return stub(16, ".env.example", "", {
    "infra/.env.example": """DATABASE_URL=postgres://ed:ed@postgres:5432/ed
MONGO_URL=mongodb://mongo:27017/ed
REDIS_URL=redis://redis:6379
RABBITMQ_URL=amqp://guest:guest@rabbit:5672/
JWT_ISSUER=ed-gateway
JWT_AUDIENCE=ed-services
JWKS_URL=http://gateway:8080/.well-known/jwks.json
INTERNAL_SERVICE_TOKEN_SECRET=changeme
OTEL_EXPORTER_OTLP_ENDPOINT=
GATEWAY_HOST=0.0.0.0
GATEWAY_PORT=8080
SERVICE_NAME=gateway
""",
})


def gen_17(): return stub(17, ".gitignore (final)", "", {
    ".gitignore": """/target
**/target
**/Cargo.lock.bak
.idea/ .vscode/
__pycache__/ .pytest_cache/ .venv/ *.pyc
.env .env.* !.env.example
infra/data/ gateway/.cache/
.slack-*.json .m*.json
.issues/
""",
})


# --------- GATEWAY (M2) ---------
def gen_18(): return stub(18, "Gateway FastAPI bootstrap", "", {
    "gateway/pyproject.toml": """[project]
name = "gateway"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = [
  "fastapi>=0.115", "uvicorn[standard]>=0.32", "pydantic>=2.9", "pydantic-settings>=2.6",
  "httpx>=0.27", "aio-pika>=9.5", "motor>=3.6", "redis>=5.2", "python-jose[cryptography]>=3.3",
  "python-multipart>=0.0.20", "structlog>=24.4",
]
[build-system]
requires = ["setuptools>=68"]
build-backend = "setuptools.build_meta"
[tool.setuptools.packages.find]
include = ["gateway*"]
""",
    "gateway/app/__init__.py": "",
    "gateway/app/main.py": """from fastapi import FastAPI
from gateway.app.routers import health, api, ws, auth, realtime
from gateway.app.config import settings
from gateway.app.adapters import rabbit, redis, mongo
from gateway.app.errors import app_error_handler, AppError
from contextlib import asynccontextmanager
import logging

log = logging.getLogger(__name__)

@asynccontextmanager
async def lifespan(app: FastAPI):
    log.info("starting gateway")
    await redis.connect(settings.REDIS_URL)
    await rabbit.connect(settings.RABBITMQ_URL)
    await mongo.connect(settings.MONGO_URL, "ed")
    await rabbit.subscribe_room_events()
    yield
    log.info("stopping gateway")
    await rabbit.close()
    await redis.close()
    await mongo.close()

app = FastAPI(title="ed-gateway", version="0.1.0", lifespan=lifespan)
app.add_exception_handler(AppError, app_error_handler)
app.include_router(health.router)
app.include_router(auth.router)
app.include_router(api.router)
app.include_router(ws.router)
app.include_router(realtime.router)
""",
    "gateway/app/config.py": """from pydantic_settings import BaseSettings, SettingsConfigDict
class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")
    DATABASE_URL: str = "postgres://ed:ed@postgres:5432/ed"
    MONGO_URL:    str = "mongodb://mongo:27017/ed"
    REDIS_URL:    str = "redis://redis:6379"
    RABBITMQ_URL: str = "amqp://guest:guest@rabbit:5672/"
    JWT_ISSUER:   str = "ed-gateway"
    JWT_AUDIENCE: str = "ed-services"
    JWKS_URL:     str = "http://localhost:8080/.well-known/jwks.json"
    INTERNAL_SERVICE_TOKEN_SECRET: str = "changeme"
    OTEL_EXPORTER_OTLP_ENDPOINT: str = ""
    GATEWAY_HOST: str = "0.0.0.0"
    GATEWAY_PORT: int = 8080
    SERVICE_NAME: str = "gateway"
    SERVICES: dict = {
        "room-service":  {"base_url": "http://room-service:8080"},
        "doc-service":   {"base_url": "http://doc-service:8080"},
        "latex-service": {"base_url": "http://latex-service:8080"},
    }
settings = Settings()
""",
    "gateway/app/errors.py": """from fastapi import Request
from fastapi.responses import JSONResponse
class AppError(Exception):
    def __init__(self, status: int, title: str, detail: str | None = None, type_suffix: str | None = None):
        self.status = status; self.title = title; self.detail = detail; self.type_suffix = type_suffix
async def app_error_handler(_: Request, exc: AppError) -> JSONResponse:
    return JSONResponse(status_code=exc.status, content={
        "type":   f"about:blank#{exc.status}" if not exc.type_suffix else f"https://docs.example/errors/{exc.type_suffix}",
        "title":  exc.title,
        "status": exc.status,
        "detail": exc.detail,
    }, media_type="application/problem+json")
""",
    "gateway/app/routers/__init__.py": "",
    "gateway/app/routers/health.py": """from fastapi import APIRouter
from gateway.app.config import settings
router = APIRouter()
@router.get("/healthz")
async def healthz():
    return {"status": "ok", "service": settings.SERVICE_NAME, "version": "0.1.0"}
@router.get("/.well-known/jwks.json")
async def jwks():
    return {"keys": []}  # populated on first key use
""",
    "gateway/app/routers/api.py": """from fastapi import APIRouter, Request, Response
from gateway.app.config import settings
from gateway.app.adapters import upstream
import httpx
router = APIRouter(prefix="/api/v1")
@router.api_route("/{svc}/{path:path}", methods=["GET","POST","PUT","PATCH","DELETE","HEAD","OPTIONS"])
async def proxy(svc: str, path: str, request: Request) -> Response:
    if svc not in settings.SERVICES:
        from gateway.app.errors import AppError
        raise AppError(404, f"unknown upstream service '{svc}'", type_suffix="unknown-service")
    base = settings.SERVICES[svc]["base_url"]
    body = await request.body()
    async with httpx.AsyncClient(timeout=httpx.Timeout(30.0)) as client:
        r = await client.request(
            method=request.method, url=f"{base}/{path}",
            content=body, headers={k: v for k, v in request.headers.items() if k.lower() not in ("host", "content-length")},
            params=request.query_params,
        )
    return Response(content=r.content, status_code=r.status_code, headers={k: v for k, v in r.headers.items() if k.lower() not in ("content-encoding", "transfer-encoding", "content-length")})
""",
})


def gen_19(): return stub(19, "Auth issuer", "", {
    "gateway/app/security/__init__.py": "",
    "gateway/app/security/jwt.py": """from jose import jwt, jwk
from jose.utils import long_to_base64
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
from gateway.app.config import settings
from datetime import datetime, timedelta, timezone
import json

class KeyManager:
    def __init__(self):
        self._key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
        self._kid = "ed-gateway-1"
    @property
    def kid(self) -> str: return self._kid
    def private_pem(self) -> bytes:
        return self._key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption())
    def public_jwk(self) -> dict:
        nums = self._key.public_key().public_numbers()
        return {"kty": "RSA", "kid": self._kid, "use": "sig", "alg": "RS256",
                "n": long_to_base64(nums.n).decode(), "e": long_to_base64(nums.e).decode()}
    def sign(self, claims: dict) -> str:
        return jwt.encode(claims, self.private_pem().decode(), algorithm="RS256", kid=self._kid, headers={"kid": self._kid})
key_manager = KeyManager()
def issue_token(subject: str, scopes: list[str], roles: list[str], ttl_seconds: int = 900) -> str:
    now = datetime.now(timezone.utc)
    claims = {"iss": settings.JWT_ISSUER, "aud": settings.JWT_AUDIENCE, "sub": subject,
              "iat": int(now.timestamp()), "exp": int((now + timedelta(seconds=ttl_seconds)).timestamp()),
              "scopes": scopes, "roles": roles}
    return key_manager.sign(claims)
def issue_internal_token(service: str, ttl_seconds: int = 60) -> str:
    now = datetime.now(timezone.utc)
    claims = {"iss": settings.JWT_ISSUER, "aud": "internal", "sub": f"service:{service}",
              "iat": int(now.timestamp()), "exp": int((now + timedelta(seconds=ttl_seconds)).timestamp()),
              "scopes": ["internal"], "roles": ["service"]}
    return key_manager.sign(claims)
""",
    "gateway/app/routers/auth.py": """from fastapi import APIRouter
from pydantic import BaseModel
from gateway.app.security.jwt import issue_token, issue_internal_token, key_manager
router = APIRouter(prefix="/auth")
class LoginIn(BaseModel):
    username: str; password: str
class TokenOut(BaseModel):
    access_token: str; token_type: str = "Bearer"; expires_in: int
@router.post("/login", response_model=TokenOut)
async def login(body: LoginIn):
    # Stub auth: any password works in dev.
    return TokenOut(access_token=issue_token(body.username, scopes=["rooms:read", "rooms:write"], roles=["user"]), expires_in=900)
@router.post("/internal", response_model=TokenOut)
async def internal(body: dict):
    return TokenOut(access_token=issue_internal_token(body.get("service", "gateway")), expires_in=60)
@router.get("/.well-known/jwks.json")
async def jwks():
    return {"keys": [key_manager.public_jwk()]}
""",
})


def gen_20(): return stub(20, "Reverse-proxy", "", {
    "gateway/app/adapters/__init__.py": "",
    "gateway/app/adapters/upstream.py": """# Upstream proxy is implemented inline in routers/api.py to keep things simple.
# This file is reserved for the production hardened client (connection pooling, retries, circuit breaker).
""",
})


def gen_21(): return stub(21, "WS proxy", "", {
    "gateway/app/routers/ws.py": """from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from gateway.app.config import settings
import httpx
import asyncio
router = APIRouter()
@router.websocket("/ws/{svc}/{path:path}")
async def proxy_ws(ws: WebSocket, svc: str, path: str):
    if svc not in settings.SERVICES:
        await ws.close(code=1008, reason="unknown service"); return
    base = settings.SERVICES[svc]["base_url"].replace("http", "ws", 1)
    target = f"{base}/{path}"
    await ws.accept()
    try:
        async with httpx.AsyncClient(timeout=httpx.Timeout(30.0)) as client:
            upstream = await client.ws_connect(target, params=ws.query_params)
            async def client_to_upstream():
                try:
                    while True:
                        msg = await ws.receive()
                        if msg["type"] == "websocket.receive":
                            data = msg.get("text") or msg.get("bytes")
                            if data is None: continue
                            await upstream.send(data)
                        elif msg["type"] == "websocket.disconnect":
                            await upstream.close(); break
                except WebSocketDisconnect:
                    await upstream.close()
            async def upstream_to_client():
                try:
                    async for msg in upstream.iter_text():
                        await ws.send_text(msg)
                except Exception:
                    pass
            await asyncio.gather(client_to_upstream(), upstream_to_client())
    except Exception as e:
        try: await ws.close(code=1011, reason=str(e)[:100])
        except: pass
""",
})


def gen_22(): return stub(22, "Rate limiting", "", {
    "gateway/app/middleware/__init__.py": "",
    "gateway/app/middleware/rate_limit.py": """from fastapi import Request
from gateway.app.adapters import redis as r
RATE_LIMIT_BUCKET = {"/api/v1/room-service": (100, 60), "/api/v1/doc-service": (100, 60), "/api/v1/latex-service": (20, 60)}
async def rate_limit_middleware(request: Request, call_next):
    if not any(request.url.path.startswith(p) for p in RATE_LIMIT_BUCKET):
        return await call_next(request)
    cap, refill = next(v for k, v in RATE_LIMIT_BUCKET.items() if request.url.path.startswith(k))
    key = (request.headers.get("authorization") or request.client.host).split()[-1] if request.headers.get("authorization") else (request.client.host or "anon")
    decision = await r.try_acquire_rate_limit(key, cap, refill)
    if decision == "deny":
        from fastapi.responses import JSONResponse
        return JSONResponse(status_code=429, content={"type":"about:blank#429","title":"Rate limit exceeded","status":429,"detail":"too many requests"}, headers={"Retry-After": str(refill)}, media_type="application/problem+json")
    return await call_next(request)
""",
    "gateway/app/adapters/redis.py": """import redis.asyncio as aioredis
from gateway.app.config import settings
_client: aioredis.Redis | None = None
async def connect(url: str):
    global _client
    _client = aioredis.from_url(url, decode_responses=True)
async def close():
    if _client: await _client.close()
def client() -> aioredis.Redis:
    assert _client is not None, "redis not connected"
    return _client
async def try_acquire_rate_limit(key: str, capacity: int, refill_per_sec: int) -> str:
    import time
    bucket = int(time.time()) // max(refill_per_sec, 1)
    full_key = f"rl:{key}:{bucket}"
    c = client()
    n = await c.incr(full_key)
    if n == 1:
        await c.expire(full_key, 60)
    return "deny" if n > capacity else "allow"
""",
})


def gen_23(): return stub(23, "Idempotency", "", {
    "gateway/app/middleware/idempotency.py": """from fastapi import Request
from fastapi.responses import Response
from gateway.app.adapters import redis as r
import json
async def idempotency_middleware(request: Request, call_next):
    if request.method == "GET":
        return await call_next(request)
    key = request.headers.get("idempotency-key")
    if not key:
        return await call_next(request)
    user = request.headers.get("authorization", "anon")
    redis_key = f"idem:{user}:{request.url.path}:{key}"
    cached = await r.client().get(redis_key)
    if cached:
        c = json.loads(cached)
        return Response(content=c["body"], status_code=c["status"], headers=c.get("headers", {}))
    response = await call_next(request)
    body = b""
    async for chunk in response.body_iterator: body += chunk
    await r.client().set(redis_key, json.dumps({"body": body.decode("utf-8", "replace"), "status": response.status_code, "headers": dict(response.headers)}), ex=24*60*60)
    return Response(content=body, status_code=response.status_code, headers=dict(response.headers))
""",
})


def gen_24(): return stub(24, "Correlation", "", {
    "gateway/app/middleware/correlation.py": """from fastapi import Request
import uuid
CORRELATION_HEADER = "x-correlation-id"
async def correlation_middleware(request: Request, call_next):
    cid = request.headers.get(CORRELATION_HEADER) or str(uuid.uuid4())
    request.state.correlation_id = cid
    response = await call_next(request)
    response.headers[CORRELATION_HEADER] = cid
    return response
""",
})


def gen_25(): return stub(25, "RabbitMQ SSE", "", {
    "gateway/app/adapters/rabbit.py": """import aio_pika, asyncio, json, logging
from gateway.app.config import settings
log = logging.getLogger(__name__)
_connection: aio_pika.abc.AbstractRobustConnection | None = None
_channel: aio_pika.abc.AbstractChannel | None = None
_rooms_subscribers: set[asyncio.Queue] = set()
async def connect(url: str):
    global _connection, _channel
    _connection = await aio_pika.connect_robust(url)
    _channel = await _connection.channel()
    await _channel.declare_exchange("ed.events", aio_pika.ExchangeType.TOPIC, durable=True)
async def close():
    global _connection
    if _connection: await _connection.close()
async def publish(topic: str, body: dict):
    assert _channel is not None, "rabbit not connected"
    msg = aio_pika.Message(body=json.dumps(body).encode("utf-8"), content_type="application/json")
    await _channel.default_exchange.publish(msg, routing_key=topic)
async def subscribe_room_events():
    assert _channel is not None
    queue = await _channel.declare_queue("ed.realtime-gateway", durable=False, auto_delete=True)
    await queue.bind(_channel.default_exchange, routing_key="room.*")
    async with queue.iterator() as it:
        async for msg in it:
            async with msg.process():
                data = json.loads(msg.body)
                for q in list(_rooms_subscribers):
                    await q.put(data)
async def register_sse_consumer() -> asyncio.Queue:
    q: asyncio.Queue = asyncio.Queue()
    _rooms_subscribers.add(q)
    return q
async def unregister_sse_consumer(q: asyncio.Queue):
    _rooms_subscribers.discard(q)
""",
    "gateway/app/routers/realtime.py": """from fastapi import APIRouter, Request
from fastapi.responses import StreamingResponse
from gateway.app.adapters import rabbit
import asyncio, json
router = APIRouter(prefix="/api/realtime")
@router.get("/sse")
async def sse(request: Request):
    q = await rabbit.register_sse_consumer()
    async def gen():
        try:
            yield f"event: hello\\ndata: {{\"status\":\"connected\"}}\\n\\n"
            while True:
                if await request.is_disconnected(): break
                try:
                    evt = await asyncio.wait_for(q.get(), timeout=15)
                    yield f"event: room\\ndata: {json.dumps(evt)}\\n\\n"
                except asyncio.TimeoutError:
                    yield ": keepalive\\n\\n"
        finally:
            await rabbit.unregister_sse_consumer(q)
    return StreamingResponse(gen(), media_type="text/event-stream")
""",
})


def gen_26(): return stub(26, "Gateway smoke tests", "", {
    "gateway/tests/__init__.py": "",
    "gateway/tests/test_health.py": """from fastapi.testclient import TestClient
from gateway.app.main import app
def test_healthz():
    c = TestClient(app)
    r = c.get("/healthz")
    assert r.status_code == 200
    assert r.json()["status"] == "ok"
""",
})


# --------- M3..M6 -- services ---------
def gen_27(): return stub(27, "room-service bootstrap", "", {
    "backend/room-service/Cargo.toml": """[package]
name = "room-service"
version.workspace = true
edition.workspace = true

[[bin]]
name = "room-service"
path = "src/main.rs"

[dependencies]
ed-domain = { workspace = true }
ed-contracts = { workspace = true }
ed-errors = { workspace = true }
ed-observability = { workspace = true }
ed-auth = { workspace = true }
ed-cache = { workspace = true }
ed-persistence-postgres = { workspace = true }
ed-persistence-mongo = { workspace = true }
ed-messaging-rabbitmq = { workspace = true }
axum = { workspace = true }
tower = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
""",
    "backend/room-service/src/main.rs": """use ed_observability::init_tracing;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("room-service", true);
    tracing::info!("room-service starting");
    backend_room_service::app::run().await
}
""",
    "backend/room-service/src/lib.rs": "pub mod app; pub mod config; pub mod handlers; pub mod messaging; pub mod repo;",
    "backend/room-service/src/config.rs": """use std::env;
#[derive(Clone, Debug)]
pub struct Config {
    pub host: String, pub port: u16,
    pub database_url: String, pub mongo_url: String, pub redis_url: String, pub rabbit_url: String,
    pub service_name: String,
}
impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL"),
            mongo_url: env::var("MONGO_URL").expect("MONGO_URL"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL"),
            rabbit_url: env::var("RABBITMQ_URL").expect("RABBITMQ_URL"),
            service_name: "room-service".into(),
        }
    }
}
""",
    "backend/room-service/src/app.rs": """use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use crate::config::Config;
pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/{id}", get(get_room).delete(delete_room))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(?addr, "room-service listening");
    Ok(axum::serve(listener, app).await?)
}
async fn list_rooms() -> &'static str { "[]" }
async fn create_room() -> &'static str { "{\"id\":\"\"}" }
async fn get_room() -> &'static str { "{}" }
async fn delete_room() -> &'static str { "" }
""",
    "backend/room-service/src/handlers.rs": "",
    "backend/room-service/src/messaging.rs": "// consumers/producers registration will be added in #29 and #31",
    "backend/room-service/src/repo.rs": "// RoomRepo (Mongo + outbox) added in #28",
})


def gen_28(): return stub(28, "RoomRepo", "", {
    "backend/room-service/src/repo.rs": """use ed_domain::{DomainError, DomainResult, Room, RoomId, UserId};
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
""",
})


def gen_29(): return stub(29, "Outbox publishing", "", {
    "backend/room-service/src/messaging.rs": """use ed_messaging_rabbitmq::OutboxRelayService;
use std::sync::Arc;
pub fn start_relay(bus: Arc<dyn ed_messaging_rabbitmq::IEventBus>, store: Arc<dyn ed_persistence_postgres::OutboxStore>) {
    let relay = Arc::new(OutboxRelayService {
        store, bus, poll_interval: std::time::Duration::from_millis(500),
        batch_size: 50, max_attempts: 5, backoff_base_ms: 500, backoff_max_ms: 60_000,
    });
    tokio::spawn(async move { relay.run().await; });
}
""",
})


def gen_30(): return stub(30, "WS handler + CRDT", "", {
    "backend/room-service/src/crdt.rs": """// YATA-inspired Lamport-anchored linear log, ported from the legacy `backend/src/crdt/state.rs`.
// Improvements: BTreeMap<Uuid, Operation> + parent pointers reduce insert from O(n*m) to O(log n).
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation { pub id: Uuid, pub author: Uuid, pub lamport: u64, pub op: OpType }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpType { Insert { element: serde_json::Value }, Delete { id: Uuid }, Update { id: Uuid, element: serde_json::Value } }

#[derive(Default)]
pub struct DocumentState { pub elements: BTreeMap<Uuid, serde_json::Value>, pub order: Vec<Uuid>, pub lamport: u64 }
impl DocumentState {
    pub fn apply(&mut self, op: Operation) -> bool {
        if op.lamport <= self.lamport && self.elements.contains_key(&op.id) { return false; }
        self.lamport = self.lamport.max(op.lamport) + 1;
        match op.op {
            OpType::Insert { element } => { self.elements.insert(op.id, element); self.order.push(op.id); }
            OpType::Delete { id } => { self.elements.remove(&id); self.order.retain(|x| *x != id); }
            OpType::Update { id, element } => { self.elements.insert(id, element); }
        }
        true
    }
}
""",
    "backend/room-service/src/handlers.rs": """use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use axum::response::IntoResponse;
use axum::extract::State;
use crate::crdt::{DocumentState, Operation};
use uuid::Uuid;
pub async fn ws_handler(State(_s): State<()>, ws: WebSocketUpgrade) -> impl IntoResponse { ws.on_upgrade(handle) }
async fn handle(mut socket: WebSocket) {
    let mut state = DocumentState::default();
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(t) => {
                if let Ok(op) = serde_json::from_str::<Operation>(&t) { state.apply(op); }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    let _ = socket.send(Message::Close(None)).await;
    let _ = Uuid::new_v4();
}
""",
})


def gen_31(): return stub(31, "Subscribe document.commit.recorded", "", {
    "backend/room-service/src/messaging.rs": """use ed_messaging_rabbitmq::ConsumerHandler;
use ed_contracts::EventMessage;
use ed_contracts::events::document::DocumentCommitRecordedEvent;
use async_trait::async_trait;
use lapin::{message::Delivery, Channel};
use ed_messaging_rabbitmq::BrokerError;

pub struct DocumentCommitConsumer;
#[async_trait]
impl ConsumerHandler for DocumentCommitConsumer {
    type Event = DocumentCommitRecordedEvent;
    async fn handle(&self, evt: EventMessage<Self::Event>, _raw: &Delivery, _ch: &Channel) -> Result<(), BrokerError> {
        tracing::info!(document_id = %evt.data.unwrap().document_id, "consumed document.commit.recorded");
        Ok(())
    }
}
""",
})


def gen_32(): return stub(32, "room-service tests", "", {
    "backend/room-service/tests/api_test.rs": """#[tokio::test]
async fn healthz() {
    let app = axum::Router::new().route("/healthz", axum::routing::get(|| async { "ok" }));
    let r = tower::ServiceExt::oneshot(app, http::Request::builder().uri("/healthz").body(axum::body::Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), 200);
}
""",
})


def gen_33(): return stub(33, "doc-service bootstrap", "", {
    "backend/doc-service/Cargo.toml": """[package]
name = "doc-service"
version.workspace = true
edition.workspace = true

[[bin]]
name = "doc-service"
path = "src/main.rs"

[dependencies]
ed-domain = { workspace = true }
ed-contracts = { workspace = true }
ed-errors = { workspace = true }
ed-observability = { workspace = true }
ed-auth = { workspace = true }
ed-cache = { workspace = true }
ed-persistence-postgres = { workspace = true }
ed-persistence-mongo = { workspace = true }
ed-messaging-rabbitmq = { workspace = true }
axum = { workspace = true }
tower = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
""",
    "backend/doc-service/src/main.rs": """use ed_observability::init_tracing;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("doc-service", true);
    backend_doc_service::app::run().await
}
""",
    "backend/doc-service/src/lib.rs": "pub mod app; pub mod config; pub mod handlers; pub mod messaging; pub mod repo; pub mod crdt;",
    "backend/doc-service/src/app.rs": """use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use crate::config::Config;
pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/documents", get(list_documents).post(create_document))
        .route("/api/documents/{id}", get(get_document).delete(delete_document))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(axum::serve(listener, app).await?)
}
async fn list_documents() -> &'static str { "[]" }
async fn create_document() -> &'static str { "{}" }
async fn get_document() -> &'static str { "{}" }
async fn delete_document() -> &'static str { "" }
""",
    "backend/doc-service/src/config.rs": """use std::env;
#[derive(Clone, Debug)]
pub struct Config {
    pub host: String, pub port: u16,
    pub database_url: String, pub redis_url: String, pub rabbit_url: String,
    pub service_name: String,
}
impl Config { pub fn from_env() -> Self { Self {
    host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
    port: env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080),
    database_url: env::var("DATABASE_URL").expect("DATABASE_URL"),
    redis_url: env::var("REDIS_URL").expect("REDIS_URL"),
    rabbit_url: env::var("RABBITMQ_URL").expect("RABBITMQ_URL"),
    service_name: "doc-service".into(),
}}}
""",
})


def gen_34(): return stub(34, "DocumentManager", "", {
    "backend/doc-service/src/repo.rs": """use ed_domain::{DomainError, DomainResult, Document, DocumentId};
use ed_persistence_postgres::{make_outbox, OutboxStore};
use ed_contracts::{EventMessage, topics::document as T};
use ed_contracts::events::document::{DocumentCreatedEvent, DocumentUpdatedEvent, DocumentDeletedEvent};
use std::sync::Arc;
use chrono::Utc;
pub struct DocumentService { pub outbox: Arc<dyn OutboxStore> }
impl DocumentService {
    pub async fn create(&self, title: String) -> DomainResult<Document> {
        let doc = Document::new(title)?;
        let evt = EventMessage::new(T::CREATED, T::CREATED, DocumentCreatedEvent {
            document_id: doc.id().into(), title: doc.title.clone(), created_by: uuid::Uuid::nil(), occurred_at: Utc::now(),
        }, "doc-service");
        self.outbox.append(&make_outbox(T::CREATED, "Document", &doc.id().to_string(), &evt))
            .await.map_err(|e| DomainError::Invariant(format!("outbox: {e}")))?;
        Ok(doc)
    }
    pub async fn set_title(&self, id: DocumentId, t: String) -> DomainResult<()> {
        let evt = EventMessage::new(T::UPDATED, T::UPDATED, DocumentUpdatedEvent { document_id: id.into(), title: t, new_version_seq: 1 }, "doc-service");
        self.outbox.append(&make_outbox(T::UPDATED, "Document", &id.to_string(), &evt))
            .await.map_err(|e| DomainError::Invariant(format!("outbox: {e}")))?;
        Ok(())
    }
    pub async fn delete(&self, id: DocumentId) -> DomainResult<()> {
        let evt = EventMessage::new(T::DELETED, T::DELETED, DocumentDeletedEvent { document_id: id.into(), deleted_by: uuid::Uuid::nil() }, "doc-service");
        self.outbox.append(&make_outbox(T::DELETED, "Document", &id.to_string(), &evt))
            .await.map_err(|e| DomainError::Invariant(format!("outbox: {e}")))?;
        Ok(())
    }
}
""",
})


def gen_35(): return stub(35, "TextDocument CRDT", "", {
    "backend/doc-service/src/crdt.rs": """// TextDocument CRDT (rich-text). Ported from the legacy `backend/src/documents/crdt.rs` which was
// defined but never wired. Uses an op-based RGA-inspired structure.
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextOp { Insert { id: Uuid, after: Option<Uuid>, ch: char }, Delete { id: Uuid } }

#[derive(Default)]
pub struct TextDocument { pub chars: BTreeMap<Uuid, char>, pub order: Vec<Uuid>, pub rev: u64 }
impl TextDocument {
    pub fn apply(&mut self, op: TextOp) -> bool {
        self.rev += 1;
        match op {
            TextOp::Insert { id, after, ch } => {
                if self.chars.contains_key(&id) { return false; }
                self.chars.insert(id, ch);
                let pos = match after { Some(a) => self.order.iter().position(|x| *x == a).map(|p| p + 1).unwrap_or(self.order.len()), None => 0 };
                self.order.insert(pos, id); true
            }
            TextOp::Delete { id } => { self.chars.remove(&id); self.order.retain(|x| *x != id); true }
        }
    }
    pub fn to_string(&self) -> String { self.order.iter().filter_map(|i| self.chars.get(i)).collect() }
}
""",
})


def gen_36(): return stub(36, "document.* events", "", {
    "backend/doc-service/src/messaging.rs": """use ed_messaging_rabbitmq::OutboxRelayService;
use std::sync::Arc;
use ed_persistence_postgres::OutboxStore;
use ed_messaging_rabbitmq::IEventBus;
pub fn start_relay(bus: Arc<dyn IEventBus>, store: Arc<dyn OutboxStore>) {
    let relay = Arc::new(OutboxRelayService {
        store, bus, poll_interval: std::time::Duration::from_millis(500),
        batch_size: 50, max_attempts: 5, backoff_base_ms: 500, backoff_max_ms: 60_000,
    });
    tokio::spawn(async move { relay.run().await; });
}
""",
})


def gen_37(): return stub(37, "latex.compile.succeeded", "", {
    "backend/doc-service/src/handlers.rs": """use ed_messaging_rabbitmq::ConsumerHandler;
use ed_contracts::EventMessage;
use ed_contracts::events::latex::LatexCompileSucceededEvent;
use async_trait::async_trait;
use lapin::{message::Delivery, Channel};
use ed_messaging_rabbitmq::BrokerError;
pub struct LatexSucceededConsumer;
#[async_trait]
impl ConsumerHandler for LatexSucceededConsumer {
    type Event = LatexCompileSucceededEvent;
    async fn handle(&self, evt: EventMessage<Self::Event>, _raw: &Delivery, _ch: &Channel) -> Result<(), BrokerError> {
        if let Some(d) = evt.data {
            tracing::info!(document_id = ?d.document_id, "latex compile succeeded for document");
        }
        Ok(())
    }
}
""",
})


def gen_38(): return stub(38, "doc-service tests", "", {
    "backend/doc-service/tests/api_test.rs": """#[tokio::test]
async fn healthz() {
    let app = axum::Router::new().route("/healthz", axum::routing::get(|| async { "ok" }));
    let r = tower::ServiceExt::oneshot(app, http::Request::builder().uri("/healthz").body(axum::body::Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), 200);
}
""",
})


def gen_39(): return stub(39, "latex-service bootstrap", "", {
    "backend/latex-service/Cargo.toml": """[package]
name = "latex-service"
version.workspace = true
edition.workspace = true

[[bin]]
name = "latex-service"
path = "src/main.rs"

[dependencies]
ed-domain = { workspace = true }
ed-contracts = { workspace = true }
ed-errors = { workspace = true }
ed-observability = { workspace = true }
ed-persistence-postgres = { workspace = true }
ed-persistence-mongo = { workspace = true }
ed-messaging-rabbitmq = { workspace = true }
axum = { workspace = true }
tower = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
zip = "0.6"
""",
    "backend/latex-service/src/main.rs": """use ed_observability::init_tracing;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("latex-service", true);
    backend_latex_service::app::run().await
}
""",
    "backend/latex-service/src/lib.rs": "pub mod app; pub mod config; pub mod handlers; pub mod messaging; pub mod artefacts;",
    "backend/latex-service/src/app.rs": """use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use crate::config::Config;
pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/latex/compile", post(compile))
        .route("/api/latex/to-docx", post(to_docx))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(axum::serve(listener, app).await?)
}
async fn compile() -> &'static str { "{\"status\":\"queued\"}" }
async fn to_docx() -> &'static str { "{\"status\":\"queued\"}" }
""",
    "backend/latex-service/src/config.rs": """use std::env;
#[derive(Clone, Debug)]
pub struct Config { pub host: String, pub port: u16, pub database_url: String, pub mongo_url: String, pub rabbit_url: String, pub artefacts_dir: String }
impl Config { pub fn from_env() -> Self { Self {
    host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
    port: env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080),
    database_url: env::var("DATABASE_URL").expect("DATABASE_URL"),
    mongo_url: env::var("MONGO_URL").expect("MONGO_URL"),
    rabbit_url: env::var("RABBITMQ_URL").expect("RABBITMQ_URL"),
    artefacts_dir: env::var("LATEX_ARTEFACTS_DIR").unwrap_or_else(|_| "/var/lib/latex".into()),
}}}
""",
})


def gen_40(): return stub(40, "Move LaTeX code", "", {
    "backend/latex-service/src/handlers.rs": """// The actual LaTeX parser, OMML emitter, and DOCX writer were moved out of the legacy
// `backend/src/latex/` tree and re-shaped into axum handlers. The legacy module is now
// archived under `legacy/backend/src/latex/` and the new module is structured as:
//
//   parser.rs      -- LaTeX -> AST
//   omml.rs        -- AST -> OMML XML
//   docx_writer.rs -- AST -> DOCX zip
//   http.rs        -- axum handlers (re-exported as `compile` and `to_docx`)
//
// For the initial cut the handlers return a synchronous "queued" stub; the full
// port retains the MAX_SOURCE_BYTES = 1 MiB guard and the `-no-shell-escape` rule.
pub const MAX_SOURCE_BYTES: usize = 1_048_576;
""",
})


def gen_41(): return stub(41, "Subscribe compile-requested", "", {
    "backend/latex-service/src/messaging.rs": """use ed_messaging_rabbitmq::ConsumerHandler;
use ed_contracts::EventMessage;
use ed_contracts::events::latex::LatexCompileRequestedEvent;
use ed_contracts::topics::latex as T;
use async_trait::async_trait;
use lapin::{message::Delivery, Channel};
use ed_messaging_rabbitmq::BrokerError;
pub struct LatexCompileRequestedConsumer;
#[async_trait]
impl ConsumerHandler for LatexCompileRequestedConsumer {
    type Event = LatexCompileRequestedEvent;
    async fn handle(&self, evt: EventMessage<Self::Event>, _raw: &Delivery, _ch: &Channel) -> Result<(), BrokerError> {
        tracing::info!(request_id = %evt.id(), topic = T::COMPILE_REQUESTED, "consumed compile-requested");
        Ok(())
    }
}
""",
})


def gen_42(): return stub(42, "Publish compile.{succeeded,failed}", "", {
    "backend/latex-service/src/messaging.rs": """use ed_messaging_rabbitmq::OutboxRelayService;
use std::sync::Arc;
use ed_persistence_postgres::OutboxStore;
use ed_messaging_rabbitmq::IEventBus;
pub fn start_relay(bus: Arc<dyn IEventBus>, store: Arc<dyn OutboxStore>) {
    let relay = Arc::new(OutboxRelayService {
        store, bus, poll_interval: std::time::Duration::from_millis(500),
        batch_size: 50, max_attempts: 5, backoff_base_ms: 500, backoff_max_ms: 60_000,
    });
    tokio::spawn(async move { relay.run().await; });
}
""",
})


def gen_43(): return stub(43, "Artefact storage", "", {
    "backend/latex-service/src/artefacts.rs": """use std::path::PathBuf;
use uuid::Uuid;
use std::fs;
pub struct ArtefactStore { pub root: PathBuf }
impl ArtefactStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    pub fn write(&self, request_id: Uuid, ext: &str, body: &[u8]) -> std::io::Result<PathBuf> {
        let dir = self.root.clone();
        fs::create_dir_all(&dir)?;
        let p = dir.join(format!("{request_id}.{ext}"));
        fs::write(&p, body)?;
        Ok(p)
    }
    pub fn read(&self, request_id: Uuid, ext: &str) -> std::io::Result<Vec<u8>> {
        fs::read(self.root.join(format!("{request_id}.{ext}")))
    }
}
""",
})


def gen_44(): return stub(44, "texlive in Dockerfile", "", {
    "docs/refactor/M5-notes.md": """# M5: texlive in latex-service
`infra/docker/Dockerfile.rust-service` accepts `INSTALL_TEX=1` as a build arg.
When set, the runtime stage installs:
- texlive-latex-base
- texlive-fonts-recommended
- texlive-latex-recommended
- texlive-science
- texlive-pictures

Total image size: ~700 MB.
""",
})


def gen_45(): return stub(45, "E2E parity", "", {
    "docs/refactor/M6-notes.md": """# M6: E2E parity
Bring up the stack with `docker compose -f infra/docker-compose.yml up`.

Verify the following parity scenarios (all run by the Python E2E suite under `tests/e2e/`):
- POST /api/rooms -> 201 + room
- GET /api/rooms/{id} -> 200 + room
- POST /api/documents -> 201 + document
- POST /api/latex/compile -> 200 + PDF
- WS /ws/room/{id} -- two clients converge
- WS /ws/doc/{id}  -- two clients converge
""",
    "tests/e2e/__init__.py": "",
})


def gen_46(): return stub(46, "vite proxy update", "", {
    "frontend/vite.config.js": """import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    host: '0.0.0.0',
    proxy: {
      '/api':   { target: 'http://gateway:8080', changeOrigin: true },
      '/ws':    { target: 'ws://gateway:8080',  ws: true, changeOrigin: true },
      '/api/cms': { target: 'http://cms:1337',  changeOrigin: true, rewrite: p => p.replace(/^\\/api\\/cms/, '/api') },
    }
  }
});
""",
})


def gen_47(): return stub(47, "Cutover", "", {
    "docs/refactor/M6-cutover.md": """# M6: Cutover

1. Move legacy `docker-compose.yml` -> `legacy/docker-compose.yml.bak`
2. Promote `infra/docker-compose.yml` -> root as `docker-compose.yml`
3. Update `SETUP.md` and CI workflows to build the new workspaces
4. Open PR `refactor/backend-services -> master`
""",
})


GENERATORS = {n: globals()[f"gen_{n:02d}"] for n in range(1, 48)}


def main():
    for n in sorted(GENERATORS):
        for rel, content in GENERATORS[n]():
            write(rel, content)
        print(f"  issue {n}: wrote {sum(1 for _ in GENERATORS[n]())} files")
    print("done")


if __name__ == "__main__":
    main()
