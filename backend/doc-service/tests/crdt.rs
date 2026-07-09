//! Property-based CRDT tests for the rich-text editor (doc-service).
//!
//! Run with: `cargo test -p doc-service crdt`

use backend_doc_service::crdt::{TextDocument, TextOp};
use proptest::prelude::*;
use uuid::Uuid;

#[test]
fn empty_document_renders_to_empty_string() {
    let d = TextDocument::default();
    assert_eq!(d.to_string(), "");
    assert_eq!(d.rev, 0);
}

#[test]
fn insert_then_read() {
    let mut d = TextDocument::default();
    let id = Uuid::new_v4();
    let applied = d.apply(TextOp::Insert { id, after: None, ch: 'h' });
    assert!(applied);
    let applied = d.apply(TextOp::Insert { id: Uuid::new_v4(), after: Some(id), ch: 'i' });
    assert!(applied);
    assert_eq!(d.to_string(), "hi");
    assert_eq!(d.rev, 2);
}

#[test]
fn delete_removes_character() {
    let mut d = TextDocument::default();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    d.apply(TextOp::Insert { id: a, after: None, ch: 'x' });
    d.apply(TextOp::Insert { id: b, after: Some(a), ch: 'y' });
    d.apply(TextOp::Delete { id: a });
    assert_eq!(d.to_string(), "y");
}

#[test]
fn delete_nonexistent_returns_false() {
    let mut d = TextDocument::default();
    let applied = d.apply(TextOp::Delete { id: Uuid::new_v4() });
    assert!(!applied);
    assert_eq!(d.rev, 0);
}

#[test]
fn insert_duplicate_id_returns_false() {
    let mut d = TextDocument::default();
    let id = Uuid::new_v4();
    assert!(d.apply(TextOp::Insert { id, after: None, ch: 'a' }));
    // Same id, should be rejected (idempotency)
    let applied = d.apply(TextOp::Insert { id, after: None, ch: 'b' });
    assert!(!applied);
    assert_eq!(d.to_string(), "a");
}

proptest! {
    /// Two replicas that apply the same operations in the same order
    /// must converge to identical state.
    #[test]
    fn text_crdt_converges(
        ops in proptest::collection::vec(
            prop_oneof![
                (any::<char>(), any::<bool>()).prop_map(|(c, _)| (0u8, c)),  // insert
                (0..100u32).prop_map(|i| (1u8, std::char::from_u32((b'a' as u32 + (i % 26)) as u32).unwrap_or('?')))  // insert via delete-id
            ],
            5..50
        )
    ) {
        let mut a = TextDocument::default();
        let mut b = TextDocument::default();
        for (kind, ch) in &ops {
            let op = if *kind == 0 {
                TextOp::Insert { id: Uuid::new_v4(), after: None, ch: *ch }
            } else {
                // simulate a delete of the last char (if any)
                let last = a.order.last().copied();
                match last {
                    Some(id) => TextOp::Delete { id },
                    None => continue,
                }
            };
            a.apply(op.clone());
            b.apply(op);
        }
        assert_eq!(a.to_string(), b.to_string());
        assert_eq!(a.rev, b.rev);
    }
}
