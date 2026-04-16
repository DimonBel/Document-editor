use serde::{Deserialize, Serialize};

/// Serialisable client identity (sent to peers on join/sync).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub id: String,
    pub name: String,
}
