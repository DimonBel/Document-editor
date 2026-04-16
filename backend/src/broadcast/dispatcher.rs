use actix::Message;

/// A raw JSON string to be forwarded to a connected WebSocket client.
/// Keeping it as a plain `String` avoids repeated serialisation in the hot path.
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct OutboundMessage(pub String);
