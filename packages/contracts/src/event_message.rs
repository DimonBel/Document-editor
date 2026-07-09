use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub trait IEventMessage<T> {
    fn id(&self) -> Uuid; fn occurred_at(&self) -> DateTime<Utc>;
    fn data(&self) -> Option<&T>; fn service_name(&self) -> &str; fn topic(&self) -> &str;
    fn correlation_id(&self) -> &str; fn schema_version(&self) -> &str; fn event_name(&self) -> &str;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage<T> {
    pub id: Uuid,
    #[serde(rename = "occurredAt")] pub occurred_at: DateTime<Utc>,
    pub service_name: String, pub module_id: String,
    pub event_name: String, pub topic: String,
    pub correlation_id: String, pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<T>,
}
impl<T> EventMessage<T> {
    pub fn new(topic: impl Into<String>, event: impl Into<String>, data: T, svc: impl Into<String>) -> Self {
        Self { id: Uuid::new_v4(), occurred_at: Utc::now(), service_name: svc.into(),
            module_id: String::new(), event_name: event.into(), topic: topic.into(),
            correlation_id: Uuid::new_v4().to_string(), schema_version: "1".into(), data: Some(data) }
    }
    pub fn with_correlation(mut self, c: impl Into<String>) -> Self { self.correlation_id = c.into(); self }
}
impl<T> IEventMessage<T> for EventMessage<T> {
    fn id(&self) -> Uuid { self.id } fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn data(&self) -> Option<&T> { self.data.as_ref() } fn service_name(&self) -> &str { &self.service_name }
    fn topic(&self) -> &str { &self.topic } fn correlation_id(&self) -> &str { &self.correlation_id }
    fn schema_version(&self) -> &str { &self.schema_version } fn event_name(&self) -> &str { &self.event_name }
}
