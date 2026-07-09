use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use ed_contracts::EventMessage;
use crate::error::PgError;
use async_trait::async_trait;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutboxStatus { Pending = 0, Retrying = 1, InFlight = 2, Sent = 3, DeadLettered = 4 }
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OutboxMessage {
    pub id: Uuid, pub occurred_at: DateTime<Utc>,
    pub topic: String, pub aggregate_type: String, pub aggregate_id: String,
    pub correlation_id: String, pub payload: serde_json::Value,
    pub status: i16, pub attempt_count: i32,
    pub last_error: Option<String>, pub next_attempt_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>, pub created_at: DateTime<Utc>,
}
#[async_trait]
pub trait OutboxStore: Send + Sync {
    async fn append(&self, msg: &OutboxMessage) -> Result<(), PgError>;
    async fn claim_pending(&self, limit: i64) -> Result<Vec<OutboxMessage>, PgError>;
    async fn mark_sent(&self, id: Uuid) -> Result<(), PgError>;
    async fn mark_failed(&self, id: Uuid, err: &str, backoff_secs: i64) -> Result<(), PgError>;
    async fn mark_dead_lettered(&self, id: Uuid, err: &str) -> Result<(), PgError>;
}
pub struct EfOutboxStore { pub pool: PgPool }
#[async_trait]
impl OutboxStore for EfOutboxStore {
    async fn append(&self, m: &OutboxMessage) -> Result<(), PgError> {
        sqlx::query("INSERT INTO outbox_messages (id, occurred_at, topic, aggregate_type, aggregate_id, correlation_id, payload, status, attempt_count, next_attempt_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
            .bind(m.id).bind(m.occurred_at).bind(&m.topic).bind(&m.aggregate_type).bind(&m.aggregate_id)
            .bind(&m.correlation_id).bind(&m.payload).bind(m.status).bind(m.attempt_count).bind(m.next_attempt_at)
            .execute(&self.pool).await?;
        Ok(())
    }
    async fn claim_pending(&self, limit: i64) -> Result<Vec<OutboxMessage>, PgError> {
        let rows = sqlx::query_as::<_, OutboxMessage>("SELECT id, occurred_at, topic, aggregate_type, aggregate_id, correlation_id, payload, status, attempt_count, last_error, next_attempt_at, sent_at, created_at FROM outbox_messages WHERE status IN (0,1) AND next_attempt_at <= now() ORDER BY next_attempt_at ASC LIMIT $1 FOR UPDATE SKIP LOCKED")
            .bind(limit).fetch_all(&self.pool).await?;
        Ok(rows)
    }
    async fn mark_sent(&self, id: Uuid) -> Result<(), PgError> {
        sqlx::query("UPDATE outbox_messages SET status = 3, sent_at = now() WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }
    async fn mark_failed(&self, id: Uuid, err: &str, backoff_secs: i64) -> Result<(), PgError> {
        sqlx::query("UPDATE outbox_messages SET status = 1, attempt_count = attempt_count + 1, last_error = $2, next_attempt_at = now() + ($3::int * interval '1 second') WHERE id = $1")
            .bind(id).bind(err).bind(backoff_secs as i32).execute(&self.pool).await?;
        Ok(())
    }
    async fn mark_dead_lettered(&self, id: Uuid, err: &str) -> Result<(), PgError> {
        sqlx::query("UPDATE outbox_messages SET status = 4, last_error = $2 WHERE id = $1").bind(id).bind(err).execute(&self.pool).await?;
        Ok(())
    }
}
pub fn make_outbox<T: Serialize>(topic: &str, aggregate_type: &str, aggregate_id: &str, evt: &EventMessage<T>) -> OutboxMessage {
    OutboxMessage {
        id: Uuid::new_v4(), occurred_at: Utc::now(),
        topic: topic.to_string(), aggregate_type: aggregate_type.to_string(), aggregate_id: aggregate_id.to_string(),
        correlation_id: evt.correlation_id().to_string(),
        payload: serde_json::to_value(evt).unwrap_or(serde_json::Value::Null),
        status: OutboxStatus::Pending as i16, attempt_count: 0, last_error: None,
        next_attempt_at: Utc::now(), sent_at: None, created_at: Utc::now(),
    }
}
