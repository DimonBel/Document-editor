use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecordedEvent { pub service_context: String, pub entity_type: String, pub entity_id: String, pub action: String, pub actor: String, pub payload: Value }
