use serde::{Deserialize, Serialize};
use crate::AppError;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")] pub kind: String, pub title: String, pub status: u16, pub detail: Option<String>, pub instance: Option<String>,
}
impl ProblemDetails {
    pub fn from_app(err: &AppError, instance: Option<String>) -> Self {
        let status = err.http_status();
        let title = match err {
            AppError::Domain(_) => "Domain error", AppError::Infra(_) => "Infrastructure error",
            AppError::Broker(_) => "Broker error", AppError::Auth(_) => "Unauthorized",
            AppError::Validation(_) => "Validation failed", AppError::NotFound => "Not found",
            AppError::Internal(_) => "Internal server error",
        }.to_string();
        Self { kind: format!("about:blank#{status}"), title, status, detail: Some(err.to_string()), instance }
    }
}
