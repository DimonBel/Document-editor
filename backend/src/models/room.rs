use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Serialisable room descriptor returned by the REST API.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub client_count: usize,
}
