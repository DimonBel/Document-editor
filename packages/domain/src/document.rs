use serde::{Deserialize, Serialize};
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
