//! Unit tests for `ed-domain`: entity invariants, ID newtypes, value objects.
//!
//! Run with: `cargo test -p ed-domain`

use ed_domain::{Document, DomainError, DocumentId, Room, RoomId, UserId};

// ─── Room ──────────────────────────────────────────────────────────────────
#[test]
fn room_rejects_empty_name() {
    let res = Room::new("   ".into(), UserId::new());
    assert!(matches!(res, Err(DomainError::Validation(_))));
}

#[test]
fn room_rejects_overly_long_name() {
    let long = "x".repeat(200);
    let res = Room::new(long, UserId::new());
    assert!(matches!(res, Err(DomainError::Validation(_))));
}

#[test]
fn room_accepts_valid_name() {
    let r = Room::new("Standup".into(), UserId::new()).expect("ok");
    assert_eq!(r.name, "Standup");
    assert_eq!(r.audit.entity.version, 0);
    assert!(!r.audit.is_deleted);
    assert!(r.audit.created_at.is_none());  // populated by interceptor
}

#[test]
fn room_rename_increments_version() {
    let mut r = Room::new("Standup".into(), UserId::new()).expect("ok");
    r.rename("Sprint planning".into()).expect("ok");
    assert_eq!(r.name, "Sprint planning");
    assert_eq!(r.audit.entity.version, 1);
}

#[test]
fn room_set_latex_source_bumps_version() {
    let mut r = Room::new("Standup".into(), UserId::new()).expect("ok");
    let v0 = r.audit.entity.version;
    r.set_latex_source(Some(r"\documentclass{article}".into()));
    assert_eq!(r.audit.entity.version, v0 + 1);
    assert!(r.latex_source.is_some());
}

#[test]
fn rooms_with_same_id_are_equal() {
    let id = RoomId::new();
    let a = Room { audit: ed_domain::entity::AuditableEntity::new(id), name: "A".into(), created_by: UserId::new(), latex_source: None, snapshot_seq: 0 };
    let b = Room { audit: ed_domain::entity::AuditableEntity::new(id), name: "B".into(), created_by: UserId::new(), latex_source: None, snapshot_seq: 0 };
    assert_eq!(a, b);
}

// ─── Document ──────────────────────────────────────────────────────────────
#[test]
fn document_rejects_empty_title() {
    let res = Document::new("".into());
    assert!(matches!(res, Err(DomainError::Validation(_))));
}

#[test]
fn document_set_title_validates() {
    let mut d = Document::new("Hello".into()).expect("ok");
    d.set_title("World".into()).expect("ok");
    assert_eq!(d.title, "World");
    assert_eq!(d.audit.entity.version, 1);

    let bad = d.set_title("   ".into());
    assert!(bad.is_err());
    // title should not have changed
    assert_eq!(d.title, "World");
}

#[test]
fn documents_with_different_ids_are_not_equal() {
    let a = Document::new("A".into()).unwrap();
    let b = Document::new("A".into()).unwrap();
    assert_ne!(a, b);
}

// ─── IDs ───────────────────────────────────────────────────────────────────
#[test]
fn ids_round_trip_through_uuid() {
    let u = uuid::Uuid::new_v4();
    let rid: RoomId = u.into();
    let back: uuid::Uuid = rid.into();
    assert_eq!(u, back);
}

#[test]
fn ids_are_unique() {
    let a = RoomId::new();
    let b = RoomId::new();
    assert_ne!(a, b);
    let a = DocumentId::new();
    let b = DocumentId::new();
    assert_ne!(a, b);
}

#[test]
fn ids_serialize_to_uuid_string() {
    let id = RoomId::new();
    let s = serde_json::to_string(&id).unwrap();
    // serde(transparent) -> emits the wrapped Uuid as a string
    assert!(s.contains("\""));
    let back: RoomId = serde_json::from_str(&s).unwrap();
    assert_eq!(id, back);
}

#[test]
fn client_id_serializes_as_string() {
    let id = ed_domain::ClientId::new();
    let s = serde_json::to_string(&id).unwrap();
    let back: ed_domain::ClientId = serde_json::from_str(&s).unwrap();
    assert_eq!(id, back);
}

#[test]
fn id_display_matches_uuid_display() {
    let id = RoomId::new();
    let u: uuid::Uuid = id.into();
    assert_eq!(id.to_string(), u.to_string());
}

// ─── DomainError serialisation ────────────────────────────────────────────
#[test]
fn domain_error_serialises_with_tag() {
    let e = DomainError::NotFound { entity: "Room".into(), id: "abc".into() };
    let j = serde_json::to_string(&e).unwrap();
    assert!(j.contains("\"type\":\"NotFound\""));
    let back: DomainError = serde_json::from_str(&j).unwrap();
    assert!(matches!(back, DomainError::NotFound { .. }));
}

#[test]
fn domain_error_validation_round_trips() {
    let e = DomainError::Validation("nope".into());
    let j = serde_json::to_string(&e).unwrap();
    let back: DomainError = serde_json::from_str(&j).unwrap();
    assert_eq!(format!("{e:?}"), format!("{back:?}"));
}
