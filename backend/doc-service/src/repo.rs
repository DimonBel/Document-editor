use ed_domain::{DomainError, DomainResult, Document, DocumentId};
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
