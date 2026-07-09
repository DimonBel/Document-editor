use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
macro_rules! id_newtype { ($name:ident) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct $name(pub Uuid);
    impl $name { pub fn new() -> Self { Self(Uuid::new_v4()) } }
    impl Default for $name { fn default() -> Self { Self::new() } }
    impl fmt::Display for $name { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
    impl From<Uuid> for $name { fn from(v: Uuid) -> Self { Self(v) } }
    impl From<$name> for Uuid { fn from(v: $name) -> Self { v.0 } }
}; }
id_newtype!(UserId); id_newtype!(RoomId); id_newtype!(DocumentId);
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub String);
impl ClientId { pub fn new() -> Self { Self(Uuid::new_v4().to_string()) } }
impl Default for ClientId { fn default() -> Self { Self::new() } }
impl fmt::Display for ClientId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
