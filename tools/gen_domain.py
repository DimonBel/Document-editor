#!/usr/bin/env python3
"""Generate all source files for the Document-editor refactor.

Usage: python tools/gen_sources.py

After running:
  git add -A
  git commit -m "refactor: scaffold packages, services, gateway, infra"
  git push -u origin refactor/backend-services
"""
from __future__ import annotations
from pathlib import Path
import os

ROOT = Path(r"C:\Users\dmitrii.belih\Desktop\MyProject\Document-editor")
PKG = ROOT / "packages"
SVC = ROOT / "backend"
GATEWAY = ROOT / "gateway"
INFRA = ROOT / "infra"

for d in (PKG, SVC, GATEWAY, INFRA):
    d.mkdir(parents=True, exist_ok=True)


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content.lstrip("\n"), encoding="utf-8")
    print(f"  wrote {path.relative_to(ROOT)}")


# ============================================================================
# PACKAGES
# ============================================================================
def gen_ed_domain():
    print("[ed-domain]")
    write(PKG / "domain/Cargo.toml", """
[package]
name = "ed-domain"
version.workspace = true
edition.workspace = true
description = "Pure domain types: entities, value objects, IDs, domain errors. No broker/db deps."

[dependencies]
thiserror = { workspace = true }
serde = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
""")
    write(PKG / "domain/src/lib.rs", """
//! `ed-domain` -- pure domain types.
//!
//! This crate has *no* infrastructure dependencies (no `sqlx`, `lapin`, `redis`).
//! It defines the entities, value objects, identifiers, and domain errors
//! used everywhere else in the platform.

pub mod entity;
pub mod value_object;
pub mod ids;
pub mod error;
pub mod room;
pub mod document;

pub use entity::{Entity, AuditableEntity, IRowStamped, IAggregateRoot};
pub use value_object::ValueObject;
pub use ids::{RoomId, DocumentId, UserId, ClientId};
pub use error::DomainError;
pub use room::Room;
pub use document::Document;
""")
    write(PKG / "domain/src/entity.rs", """
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use uuid::Uuid;

/// Marker for entities that have a UUID identifier.
pub trait EntityId {
    fn id(&self) -> Uuid;
}

/// Aggregate root marker. Repositories only load/persist aggregate roots.
pub trait IAggregateRoot {}

/// Row-stamp columns managed by the persistence interceptor.
pub trait IRowStamped {
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn created_by(&self) -> Option<&str>;
    fn updated_at(&self) -> Option<DateTime<Utc>>;
    fn updated_by(&self) -> Option<&str>;
    fn is_deleted(&self) -> bool;
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn deleted_by(&self) -> Option<&str>;
}

/// Generic base entity. `TId` is the typed identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity<TId: Clone> {
    pub id: TId,
    pub version: u64,
}

impl<TId: Clone + PartialEq> PartialEq for Entity<TId> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<TId: Clone + Copy + std::fmt::Debug + PartialEq> EntityId for Entity<TId>
where
    TId: Into<Uuid>,
{
    fn id(&self) -> Uuid {
        self.id.into()
    }
}

/// Auditable entity with row-stamp columns (populated by the persistence interceptor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditableEntity<TId: Clone> {
    #[serde(flatten)]
    pub entity: Entity<TId>,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<String>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<String>,
}

impl<TId: Clone> AuditableEntity<TId> {
    pub fn new(id: TId) -> Self {
        Self {
            entity: Entity { id, version: 0 },
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            is_deleted: false,
            deleted_at: None,
            deleted_by: None,
        }
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
""")
    write(PKG / "domain/src/value_object.rs", """
use std::hash::Hash;

/// Base class for value objects. Equality is based on `GetEqualityComponents`.
pub trait ValueObject: Eq + Hash + Clone {
    fn get_equality_components(&self) -> Vec<Box<dyn std::any::Any>>;
}
""")
    write(PKG / "domain/src/ids.rs", """
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Newtype wrapper for a user ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn nil() -> Self { Self(Uuid::nil()) }
}

impl Default for UserId { fn default() -> Self { Self::new() } }
impl fmt::Display for UserId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
impl From<Uuid> for UserId { fn from(v: Uuid) -> Self { Self(v) } }
impl From<UserId> for Uuid { fn from(v: UserId) -> Self { v.0 } }

/// Newtype wrapper for a room ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoomId(pub Uuid);

impl RoomId { pub fn new() -> Self { Self(Uuid::new_v4()) } }
impl Default for RoomId { fn default() -> Self { Self::new() } }
impl fmt::Display for RoomId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
impl From<Uuid> for RoomId { fn from(v: Uuid) -> Self { Self(v) } }
impl From<RoomId> for Uuid { fn from(v: RoomId) -> Self { v.0 } }

/// Newtype wrapper for a document ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentId(pub Uuid);

impl DocumentId { pub fn new() -> Self { Self(Uuid::new_v4()) } }
impl Default for DocumentId { fn default() -> Self { Self::new() } }
impl fmt::Display for DocumentId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
impl From<Uuid> for DocumentId { fn from(v: Uuid) -> Self { Self(v) } }
impl From<DocumentId> for Uuid { fn from(v: DocumentId) -> Self { v.0 } }

/// Newtype wrapper for a WebSocket client ID (transient, not persisted).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub String);

impl ClientId { pub fn new() -> Self { Self(Uuid::new_v4().to_string()) } }
impl Default for ClientId { fn default() -> Self { Self::new() } }
impl fmt::Display for ClientId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
""")
    write(PKG / "domain/src/error.rs", """
use thiserror::Error;

/// Errors raised by domain invariants. These are NEVER `Internal` -- they represent
/// the user / client making a request that violates a business rule.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum DomainError {
    #[error("validation failed: {0}")]
    Validation(String),

    #[error("not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("invariant violated: {0}")]
    Invariant(String),
}

pub type DomainResult<T> = std::result::Result<T, DomainError>;
""")
    write(PKG / "domain/src/room.rs", """
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::AuditableEntity;
use crate::error::{DomainError, DomainResult};
use crate::ids::{RoomId, UserId};

/// Room aggregate (whiteboard + chat + document pointers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    #[serde(flatten)]
    pub audit: AuditableEntity<RoomId>,
    pub name: String,
    pub created_by: UserId,
    pub latex_source: Option<String>,
    pub snapshot_seq: u64,
}

impl Room {
    pub fn new(name: String, created_by: UserId) -> DomainResult<Self> {
        if name.trim().is_empty() {
            return Err(DomainError::Validation("room name must not be empty".into()));
        }
        if name.len() > 128 {
            return Err(DomainError::Validation("room name too long (max 128)".into()));
        }
        Ok(Self {
            audit: AuditableEntity::new(RoomId::new()),
            name,
            created_by,
            latex_source: None,
            snapshot_seq: 0,
        })
    }

    pub fn id(&self) -> RoomId { self.audit.entity.id }
    pub fn rename(&mut self, new_name: String) -> DomainResult<()> {
        if new_name.trim().is_empty() {
            return Err(DomainError::Validation("room name must not be empty".into()));
        }
        self.name = new_name;
        self.audit.entity.version += 1;
        Ok(())
    }
    pub fn set_latex_source(&mut self, src: Option<String>) {
        self.latex_source = src;
        self.audit.entity.version += 1;
    }
}
""")
    write(PKG / "domain/src/document.rs", """
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::AuditableEntity;
use crate::error::{DomainError, DomainResult};
use crate::ids::DocumentId;

/// Document aggregate (rich text + CRDT ops).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    #[serde(flatten)]
    pub audit: AuditableEntity<DocumentId>,
    pub title: String,
    pub content_ref: String,  // pointer to blob / snapshot
    pub version_seq: u64,
}

impl Document {
    pub fn new(title: String, created_by: String) -> DomainResult<Self> {
        if title.trim().is_empty() {
            return Err(DomainError::Validation("document title must not be empty".into()));
        }
        Ok(Self {
            audit: AuditableEntity::new(DocumentId::new()),
            title,
            content_ref: String::new(),
            version_seq: 0,
        })
    }

    pub fn id(&self) -> DocumentId { self.audit.entity.id }
    pub fn set_title(&mut self, new_title: String) -> DomainResult<()> {
        if new_title.trim().is_empty() {
            return Err(DomainError::Validation("document title must not be empty".into()));
        }
        self.title = new_title;
        self.audit.entity.version += 1;
        Ok(())
    }
}
""")

# Run domain
gen_ed_domain()
print("done")
