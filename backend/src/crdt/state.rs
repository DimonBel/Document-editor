use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::operation::{DrawElement, OpType, Operation};

/// The authoritative canvas state for one room.
/// Uses a YATA-inspired insertion-position algorithm: operations are ordered
/// primarily by Lamport timestamp, with client_id as a tiebreaker, so every
/// peer converges to the same element order without coordination.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DocumentState {
    /// id → element (live elements only; deleted ones are removed)
    pub elements: HashMap<String, DrawElement>,
    /// Insertion-ordered list of element IDs (reflects CRDT order)
    pub order: Vec<String>,
    /// Full operation log used for position lookups and dedup
    pub operations: Vec<Operation>,
    pub lamport_clock: u64,
}

impl DocumentState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply one operation and return `true` if it changed the document.
    pub fn apply(&mut self, op: Operation) -> bool {
        // Advance Lamport clock.
        self.lamport_clock = self.lamport_clock.max(op.lamport) + 1;

        // Idempotency: skip duplicate operation IDs.
        if self.operations.iter().any(|o| o.id == op.id) {
            return false;
        }

        match &op.op {
            OpType::Insert { element } => {
                let eid = element.id.clone();
                if !self.elements.contains_key(&eid) {
                    let pos = self.insert_position(op.lamport, &op.client_id);
                    self.order.insert(pos, eid.clone());
                    self.elements.insert(eid, element.clone());
                }
            }
            OpType::Delete { id } => {
                self.elements.remove(id);
                self.order.retain(|x| x != id);
            }
            OpType::Update { id, element } => {
                if self.elements.contains_key(id) {
                    self.elements.insert(id.clone(), element.clone());
                }
            }
        }

        self.operations.push(op);
        true
    }

    /// YATA-like: find the first index in `order` whose originating Insert op
    /// has a *lower* Lamport stamp (or equal stamp but lower client_id). The
    /// new element goes right after that predecessor.
    fn insert_position(&self, lamport: u64, client_id: &str) -> usize {
        let mut pos = self.order.len();
        for (i, eid) in self.order.iter().enumerate().rev() {
            if let Some(prev) = self.operations.iter().rfind(|o| {
                if let OpType::Insert { element } = &o.op {
                    element.id == *eid
                } else {
                    false
                }
            }) {
                let comes_before = prev.lamport < lamport
                    || (prev.lamport == lamport && prev.client_id.as_str() < client_id);
                if comes_before {
                    break;
                }
                pos = i;
            }
        }
        pos
    }

    /// Return elements in document order.
    pub fn ordered_elements(&self) -> Vec<&DrawElement> {
        self.order
            .iter()
            .filter_map(|id| self.elements.get(id))
            .collect()
    }
}
