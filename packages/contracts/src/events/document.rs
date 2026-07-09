use chrono::{DateTime, Utc};
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
