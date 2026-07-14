use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;
use ed_contracts::{EventMessage, IEventMessage};
use crate::error::PgError;
use async_trait::async_trait;

/// Outbox row lifecycle states.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutboxStatus { Pending = 0, Retrying = 1, InFlight = 2, Sent = 3, DeadLettered = 4 }

/// Default lease duration: how long a row stays in `InFlight` before
/// a crashed relay is assumed and the row is auto-released back to
/// `Pending` / `Retrying`.
pub const DEFAULT_LEASE_SECS: i64 = 60;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OutboxMessage {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub topic: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub correlation_id: String,
    pub payload: serde_json::Value,
    pub status: i16,
    pub attempt_count: i32,
    pub last_error: Option<String>,
    pub next_attempt_at: DateTime<Utc>,
    pub lease_until: Option<DateTime<Utc>>,
    pub leased_to: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait OutboxStore: Send + Sync {
    async fn append(&self, msg: &OutboxMessage) -> Result<(), PgError>;
    /// Atomically reserve up to `limit` rows and return them in the
    /// same transaction. Each returned row has been advanced to
    /// `InFlight` and carries a `lease_until = now() + lease_secs`
    /// guard; rows whose lease has expired are first reaped (returned
    /// to `Pending`/`Retrying`) inside the same transaction.
    async fn claim_pending(
        &self,
        limit: i64,
        lease_secs: i64,
        relay_id: &str,
    ) -> Result<Vec<OutboxMessage>, PgError>;
    async fn mark_sent(&self, id: Uuid) -> Result<(), PgError>;
    async fn mark_failed(&self, id: Uuid, err: &str, backoff_secs: i64) -> Result<(), PgError>;
    async fn mark_dead_lettered(&self, id: Uuid, err: &str) -> Result<(), PgError>;
    /// Release the lease on a row (e.g. relay shutting down). Sets
    /// status back to `Retrying` and schedules a fresh attempt.
    async fn release_lease(&self, id: Uuid, relay_id: &str) -> Result<(), PgError>;
}

pub struct EfOutboxStore { pub pool: PgPool }

#[async_trait]
impl OutboxStore for EfOutboxStore {
    async fn append(&self, m: &OutboxMessage) -> Result<(), PgError> {
        sqlx::query(
            "INSERT INTO outbox_messages (id, occurred_at, topic, aggregate_type, aggregate_id,
                                          correlation_id, payload, status, attempt_count, next_attempt_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(m.id).bind(m.occurred_at)
        .bind(&m.topic).bind(&m.aggregate_type).bind(&m.aggregate_id)
        .bind(&m.correlation_id).bind(&m.payload)
        .bind(m.status).bind(m.attempt_count)
        .bind(m.next_attempt_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    async fn claim_pending(
        &self,
        limit: i64,
        lease_secs: i64,
        relay_id: &str,
    ) -> Result<Vec<OutboxMessage>, PgError> {
        // Per #139: a single transaction with FOR UPDATE SKIP LOCKED +
        // UPDATE ... RETURNING ensures the same row is never selected
        // by two concurrent relay instances, and the lease makes
        // crashed relays non-fatal.
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        // 1. Reap expired leases (rows stuck InFlight whose lease
        //    has passed). Move them back to Retrying with an
        //    exponential backoff so a dead relay doesn't poison the
        //    queue.
        sqlx::query(
            "UPDATE outbox_messages
             SET status = 1,
                 last_error = COALESCE(last_error, 'lease expired'),
                 attempt_count = attempt_count + 1,
                 lease_until = NULL,
                 leased_to = NULL,
                 next_attempt_at = now() + (LEAST(60, 2 ^ attempt_count) || ' seconds')::interval
             WHERE status = 2
               AND lease_until IS NOT NULL
               AND lease_until < now()"
        )
        .execute(&mut *tx)
        .await?;

        // 2. Atomically claim up to `limit` rows, advancing them to
        //    `InFlight` with a fresh lease.
        let rows = sqlx::query_as::<_, OutboxMessage>(
            "WITH cte AS (
               SELECT id FROM outbox_messages
               WHERE status IN (0, 1) AND next_attempt_at <= now()
               ORDER BY next_attempt_at ASC
               LIMIT $1
               FOR UPDATE SKIP LOCKED
             )
             UPDATE outbox_messages o
             SET status = 2,
                 lease_until = now() + ($2 || ' seconds')::interval,
                 leased_to = $3,
                 attempt_count = attempt_count + 1
             FROM cte WHERE o.id = cte.id
             RETURNING o.id, o.occurred_at, o.topic, o.aggregate_type, o.aggregate_id,
                       o.correlation_id, o.payload, o.status, o.attempt_count,
                       o.last_error, o.next_attempt_at, o.lease_until, o.leased_to,
                       o.sent_at, o.created_at"
        )
        .bind(limit)
        .bind(lease_secs.to_string())
        .bind(relay_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(rows)
    }

    async fn mark_sent(&self, id: Uuid) -> Result<(), PgError> {
        sqlx::query(
            "UPDATE outbox_messages
             SET status = 3, sent_at = now(),
                 lease_until = NULL, leased_to = NULL
             WHERE id = $1"
        )
        .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, err: &str, backoff_secs: i64) -> Result<(), PgError> {
        sqlx::query(
            "UPDATE outbox_messages
             SET status = 1,
                 attempt_count = attempt_count + 1,
                 last_error = $2,
                 next_attempt_at = now() + ($3::int * interval '1 second'),
                 lease_until = NULL,
                 leased_to = NULL
             WHERE id = $1"
        )
        .bind(id).bind(err).bind(backoff_secs as i32)
        .execute(&self.pool).await?;
        Ok(())
    }

    async fn mark_dead_lettered(&self, id: Uuid, err: &str) -> Result<(), PgError> {
        sqlx::query(
            "UPDATE outbox_messages
             SET status = 4, last_error = $2,
                 lease_until = NULL, leased_to = NULL
             WHERE id = $1"
        )
        .bind(id).bind(err).execute(&self.pool).await?;
        Ok(())
    }

    async fn release_lease(&self, id: Uuid, relay_id: &str) -> Result<(), PgError> {
        sqlx::query(
            "UPDATE outbox_messages
             SET status = 1, lease_until = NULL, leased_to = NULL,
                 next_attempt_at = now()
             WHERE id = $1 AND leased_to = $2 AND status = 2"
        )
        .bind(id).bind(relay_id).execute(&self.pool).await?;
        Ok(())
    }
}

pub fn make_outbox<T: Serialize>(
    topic: &str,
    aggregate_type: &str,
    aggregate_id: &str,
    evt: &EventMessage<T>,
) -> OutboxMessage {
    let payload = match serde_json::to_value(evt) {
        Ok(v) => v,
        Err(e) => {
            // Per #149: serialisation failures MUST be surfaced, not
            // silently coerced into a JSON null payload. We return
            // the error-shaped envelope so the relay can route the
            // dead-letter event with the original failure attached.
            serde_json::json!({
                "__ed_serde_error": e.to_string(),
                "topic": topic,
                "aggregate_type": aggregate_type,
                "aggregate_id": aggregate_id,
                "occurred_at": evt.occurred_at().to_rfc3339(),
            })
        }
    };
    OutboxMessage {
        id: Uuid::new_v4(),
        occurred_at: Utc::now(),
        topic: topic.to_string(),
        aggregate_type: aggregate_type.to_string(),
        aggregate_id: aggregate_id.to_string(),
        correlation_id: evt.correlation_id().to_string(),  // .correlation_id is a field; .to_string() via the trait via Display
        payload,
        status: OutboxStatus::Pending as i16,
        attempt_count: 0,
        last_error: None,
        next_attempt_at: Utc::now(),
        lease_until: None,
        leased_to: None,
        sent_at: None,
        created_at: Utc::now(),
    }
}
