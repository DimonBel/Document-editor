//! Unit tests for `ed-contracts`: envelope round-trips, topic catalog, event payloads.
//!
//! Run with: `cargo test -p ed-contracts`

use chrono::Utc;
use ed_contracts::{EventMessage, topics};
use ed_contracts::events::room::RoomCreatedEvent;
use ed_contracts::events::document::DocumentCreatedEvent;
use ed_contracts::events::latex::LatexCompileSucceededEvent;
use serde_json::json;
use uuid::Uuid;

// ─── Envelope round-trip ───────────────────────────────────────────────────
#[test]
fn envelope_round_trip_preserves_all_fields() {
    let id = Uuid::new_v4();
    let cid = Uuid::new_v4().to_string();
    let now = Utc::now();
    let evt: EventMessage<RoomCreatedEvent> = EventMessage {
        id,
        occurred_at: now,
        service_name: "room-service".into(),
        module_id: "rooms".into(),
        event_name: topics::room::CREATED.into(),
        topic: topics::room::CREATED.into(),
        correlation_id: cid.clone(),
        schema_version: "1".into(),
        data: Some(RoomCreatedEvent {
            room_id: id,
            name: "Sprint".into(),
            created_by: id,
            occurred_at: now,
        }),
    };

    let j = serde_json::to_string(&evt).unwrap();
    let back: EventMessage<RoomCreatedEvent> = serde_json::from_str(&j).unwrap();
    assert_eq!(back.id, id);
    assert_eq!(back.service_name, "room-service");
    assert_eq!(back.correlation_id, cid);
    assert_eq!(back.data.as_ref().unwrap().name, "Sprint");
}

#[test]
fn envelope_uses_camel_case_in_json() {
    let evt: EventMessage<serde_json::Value> = EventMessage::new("room.test", "room.test", json!({"x": 1}), "test");
    let j = serde_json::to_string(&evt).unwrap();
    assert!(j.contains("\"occurredAt\""));
    assert!(j.contains("\"serviceName\""));
    assert!(j.contains("\"correlationId\""));
    assert!(j.contains("\"schemaVersion\""));
}

#[test]
fn envelope_with_correlation_keeps_caller_id() {
    let evt: EventMessage<serde_json::Value> = EventMessage::new("t", "t", json!({}), "svc")
        .with_correlation("my-trace-123");
    assert_eq!(evt.correlation_id, "my-trace-123");
}

#[test]
fn envelope_omits_none_data_in_json() {
    let evt: EventMessage<serde_json::Value> = EventMessage {
        id: Uuid::new_v4(),
        occurred_at: Utc::now(),
        service_name: "s".into(),
        module_id: "m".into(),
        event_name: "e".into(),
        topic: "t".into(),
        correlation_id: "c".into(),
        schema_version: "1".into(),
        data: None,
    };
    let j = serde_json::to_string(&evt).unwrap();
    assert!(!j.contains("\"data\""));
}

// ─── Topics ────────────────────────────────────────────────────────────────
#[test]
fn topic_constants_match_expected_strings() {
    assert_eq!(topics::room::CREATED, "room.created");
    assert_eq!(topics::room::DELETED, "room.deleted");
    assert_eq!(topics::room::USER_JOINED, "room.user_joined");
    assert_eq!(topics::room::SNAPSHOT, "room.snapshot");

    assert_eq!(topics::document::CREATED, "document.created");
    assert_eq!(topics::document::UPDATED, "document.updated");
    assert_eq!(topics::document::COMMIT_RECORDED, "document.commit_recorded");

    assert_eq!(topics::latex::COMPILE_REQUESTED, "latex.compile_requested");
    assert_eq!(topics::latex::COMPILE_SUCCEEDED, "latex.compile_succeeded");
    assert_eq!(topics::latex::DOCX_GENERATED, "latex.docx_generated");
}

#[test]
fn topic_for_uses_lowercase_concatenation() {
    assert_eq!(topics::Topics::for_ctx("Room", "User", "Created"), "room.user.created");
    assert_eq!(topics::Topics::for_ctx("Doc", "Page", "Updated"), "doc.page.updated");
}

#[test]
fn audit_recorded_uses_service_context() {
    assert_eq!(topics::audit::recorded("room"), "room.audit.recorded");
    assert_eq!(topics::audit::recorded("latex"), "latex.audit.recorded");
}

// ─── Event payloads ────────────────────────────────────────────────────────
#[test]
fn room_created_event_carries_envelope_data() {
    let id = Uuid::new_v4();
    let evt: EventMessage<RoomCreatedEvent> = EventMessage::new(
        topics::room::CREATED, topics::room::CREATED,
        RoomCreatedEvent {
            room_id: id, name: "x".into(), created_by: id, occurred_at: Utc::now(),
        },
        "room-service",
    );
    let d = evt.data.as_ref().unwrap();
    assert_eq!(d.room_id, id);
    assert_eq!(d.name, "x");
}

#[test]
fn document_event_round_trips() {
    let id = Uuid::new_v4();
    let evt: EventMessage<DocumentCreatedEvent> = EventMessage::new(
        topics::document::CREATED, topics::document::CREATED,
        DocumentCreatedEvent {
            document_id: id, title: "t".into(), created_by: id, occurred_at: Utc::now(),
        },
        "doc-service",
    );
    let j = serde_json::to_string(&evt).unwrap();
    let back: EventMessage<DocumentCreatedEvent> = serde_json::from_str(&j).unwrap();
    assert_eq!(back.data.unwrap().document_id, id);
}

#[test]
fn latex_event_carries_artefact_url() {
    let id = Uuid::new_v4();
    let evt: EventMessage<LatexCompileSucceededEvent> = EventMessage::new(
        topics::latex::COMPILE_SUCCEEDED, topics::latex::COMPILE_SUCCEEDED,
        LatexCompileSucceededEvent {
            request_id: id, document_id: None,
            pdf_artefact_url: "s3://ed/artefacts/abc.pdf".into(),
            compile_seconds: 1.234,
            source_hash: "abc123".into(),
            occurred_at: Utc::now(),
        },
        "latex-service",
    );
    let j = serde_json::to_string(&evt).unwrap();
    let back: EventMessage<LatexCompileSucceededEvent> = serde_json::from_str(&j).unwrap();
    let d = back.data.unwrap();
    assert_eq!(d.pdf_artefact_url, "s3://ed/artefacts/abc.pdf");
    assert!((d.compile_seconds - 1.234).abs() < 1e-9);
}
