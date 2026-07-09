//! `ed-domain` -- pure domain types. NO infrastructure dependencies.
pub mod entity; pub mod value_object; pub mod ids; pub mod error; pub mod room; pub mod document;
pub use entity::{Entity, AuditableEntity, IRowStamped, IAggregateRoot};
pub use value_object::ValueObject;
pub use ids::{RoomId, DocumentId, UserId, ClientId};
pub use error::{DomainError, DomainResult};
pub use room::Room;
pub use document::Document;
