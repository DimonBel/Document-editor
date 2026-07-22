CREATE TABLE IF NOT EXISTS outbox_messages (
    id              UUID PRIMARY KEY,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    topic           VARCHAR(128) NOT NULL,
    aggregate_type  VARCHAR(128) NOT NULL,
    aggregate_id    VARCHAR(128) NOT NULL,
    correlation_id  VARCHAR(64)  NOT NULL,
    payload         JSONB        NOT NULL,
    status          SMALLINT     NOT NULL DEFAULT 0,
    attempt_count   INTEGER      NOT NULL DEFAULT 0,
    last_error      VARCHAR(2048),
    next_attempt_at TIMESTAMPTZ  NOT NULL DEFAULT now(),
    lease_until     TIMESTAMPTZ,
    leased_to       VARCHAR(128),
    sent_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);
-- Backfill columns on existing installs that pre-date the lease feature.
ALTER TABLE outbox_messages ADD COLUMN IF NOT EXISTS lease_until TIMESTAMPTZ;
ALTER TABLE outbox_messages ADD COLUMN IF NOT EXISTS leased_to   VARCHAR(128);
CREATE INDEX IF NOT EXISTS ix_outbox_messages_pending ON outbox_messages (next_attempt_at) WHERE status IN (0, 1);
CREATE INDEX IF NOT EXISTS ix_outbox_messages_claim   ON outbox_messages (id)            WHERE status IN (0, 1);
CREATE INDEX IF NOT EXISTS ix_outbox_messages_sent    ON outbox_messages (sent_at)       WHERE status = 3;
-- Speeds up the reaper query inside claim_pending() (issue #247).
CREATE INDEX IF NOT EXISTS ix_outbox_messages_lease_reap
    ON outbox_messages (lease_until)
    WHERE status = 2;
