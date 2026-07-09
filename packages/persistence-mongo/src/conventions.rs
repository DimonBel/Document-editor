use chrono::{DateTime, Utc};
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
