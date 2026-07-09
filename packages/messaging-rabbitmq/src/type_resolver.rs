use lapin::types::{AMQPValue, ShortString};
use lapin::message::Delivery;
use ed_contracts::EventMessage;
use serde::de::DeserializeOwned;
pub const HEADER_TYPE_NAME: &str = "x-ed-type-name";
pub struct TypeObjectResolver;
impl TypeObjectResolver {
    pub fn decode<T: DeserializeOwned>(d: &Delivery) -> Result<EventMessage<T>, serde_json::Error> {
        serde_json::from_slice(&d.data)
    }
    pub fn get_type_name(d: &Delivery) -> Option<String> {
        d.properties.headers().as_ref()?.inner().get(HEADER_TYPE_NAME).and_then(|v| match v {
            AMQPValue::LongString(s) => Some(s.to_string()),
            AMQPValue::ShortString(s) => Some(s.to_string()),
            _ => None,
        })
    }
}
