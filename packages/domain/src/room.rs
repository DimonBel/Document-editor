use serde::{Deserialize, Serialize};
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
