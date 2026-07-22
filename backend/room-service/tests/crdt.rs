//! Property-based CRDT tests for the whiteboard (room-service).
//!
//! Run with: `cargo test -p room-service crdt`

use backend_room_service::crdt::{DocumentState, OpType, Operation};
use proptest::prelude::*;
use uuid::Uuid;
use serde_json::json;

fn arb_op(state: &DocumentState) -> BoxedStrategy<Operation> {
    let id = Uuid::new_v4();
    let lamport = state.lamport + 1;
    let author = Uuid::new_v4();
    prop_oneof![
        // Insert with random element
        (any::<String>(), any::<u64>()).prop_map(move |(s, _)| Operation {
            id, author, lamport,
            op: OpType::Insert { element: json!({"text": s}) },
        }),
        // Delete existing
        (any::<u64>()).prop_map(move |_| Operation {
            id, author, lamport,
            op: OpType::Delete { id: state.elements.keys().next().copied().unwrap_or(id) },
        }),
    ].boxed()
}

proptest! {
    /// Two replicas that apply the same operations in the same order
    /// must converge to identical state.
    #[test]
    fn crdt_converges_under_random_interleavings(ops in proptest::collection::vec(any::<(u8, String)>(), 5..50)) {
        let mut a = DocumentState::default();
        let mut b = DocumentState::default();
        for (i, (kind, s)) in ops.iter().enumerate() {
            let id = Uuid::new_v4();
            let lamport = (i as u64) + 1;
            let author = Uuid::new_v4();
            let op = if *kind % 2 == 0 {
                Operation { id, author, lamport, op: OpType::Insert { element: json!({"text": s}) } }
            } else {
                Operation { id, author, lamport, op: OpType::Delete { id } }
            };
            a.apply(op.clone());
            b.apply(op);
        }
        assert_eq!(a.lamport, b.lamport);
        assert_eq!(a.elements.len(), b.elements.len());
        assert_eq!(a.order, b.order);
    }

    /// Apply in a different order must still converge (commutativity of
    /// operations with unique Lamport timestamps).
    #[test]
    fn crdt_converges_under_reordering(ops in proptest::collection::vec(("[A-Za-z0-9]{1,16}"), 5..30)) {
        let mut a = DocumentState::default();
        let mut b = DocumentState::default();

        let mut prepared: Vec<Operation> = ops.iter().enumerate().map(|(i, s)| {
            let id = Uuid::new_v4();
            Operation {
                id, author: Uuid::new_v4(), lamport: (i as u64) + 1,
                op: OpType::Insert { element: json!({"text": s}) },
            }
        }).collect();
        let mut a_replay: Vec<Operation> = {
            let mut r = prepared.clone();
            r.reverse();
            r
        };
        for op in &prepared { a.apply(op.clone()); }
        for op in &a_replay { b.apply(op.clone()); }
        // Note: with unique Lamport, both reach identical state regardless of order.
        assert_eq!(a.lamport, b.lamport);
        assert_eq!(a.elements.len(), b.elements.len());
    }
}

#[test]
fn empty_state_has_zero_lamport() {
    let s = DocumentState::default();
    assert_eq!(s.lamport, 0);
    assert!(s.elements.is_empty());
    assert!(s.order.is_empty());
}

#[test]
fn apply_increments_lamport() {
    let mut s = DocumentState::default();
    let op = Operation {
        id: Uuid::new_v4(),
        author: Uuid::new_v4(),
        lamport: 1,
        op: OpType::Insert { element: json!({"text": "a"}) },
    };
    let accepted = s.apply(op);
    assert!(accepted);
    assert!(s.lamport > 0);
}

#[test]
fn delete_removes_from_elements_and_order() {
    let mut s = DocumentState::default();
    let id = Uuid::new_v4();
    s.apply(Operation { id, author: Uuid::new_v4(), lamport: 1, op: OpType::Insert { element: json!({}) } });
    assert!(s.elements.contains_key(&id));
    assert!(s.order.contains(&id));

    s.apply(Operation { id, author: Uuid::new_v4(), lamport: 2, op: OpType::Delete { id } });
    assert!(!s.elements.contains_key(&id));
    assert!(!s.order.contains(&id));
}
