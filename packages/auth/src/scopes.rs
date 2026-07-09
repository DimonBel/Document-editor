use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role { User, Admin, Service }
impl Role { pub fn as_str(&self) -> &'static str { match self { Role::User => "user", Role::Admin => "admin", Role::Service => "service" } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope(pub String);
impl Scope { pub fn parse(s: &str) -> Result<Self, String> { if s.is_empty() { Err("empty".into()) } else { Ok(Self(s.to_string())) } } }
