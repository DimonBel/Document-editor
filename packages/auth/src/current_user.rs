use ed_domain::UserId;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: UserId, pub email: Option<String>,
    pub roles: Vec<String>, pub scopes: Vec<String>, pub correlation_id: Option<String>,
}
impl CurrentUser {
    pub fn has_role(&self, r: &str) -> bool { self.roles.iter().any(|x| x == r) }
    pub fn has_scope(&self, s: &str) -> bool { self.scopes.iter().any(|x| x == s) }
}
