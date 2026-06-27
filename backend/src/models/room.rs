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
    /// Last persisted LaTeX source for this room. None means no
    /// LaTeX source has been set yet (clients should render the
    /// default template in this case).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latex_source: Option<String>,
}
