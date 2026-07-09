use chrono::{DateTime, Utc};
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
