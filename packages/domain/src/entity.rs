use chrono::{DateTime, Utc};
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
