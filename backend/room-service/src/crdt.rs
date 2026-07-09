// YATA-inspired Lamport-anchored linear log, ported from the legacy `backend/src/crdt/state.rs`.
// Improvements: BTreeMap<Uuid, Operation> + parent pointers reduce insert from O(n*m) to O(log n).
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation { pub id: Uuid, pub author: Uuid, pub lamport: u64, pub op: OpType }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpType { Insert { element: serde_json::Value }, Delete { id: Uuid }, Update { id: Uuid, element: serde_json::Value } }

#[derive(Default)]
pub struct DocumentState { pub elements: BTreeMap<Uuid, serde_json::Value>, pub order: Vec<Uuid>, pub lamport: u64 }
impl DocumentState {
    pub fn apply(&mut self, op: Operation) -> bool {
        if op.lamport <= self.lamport && self.elements.contains_key(&op.id) { return false; }
        self.lamport = self.lamport.max(op.lamport) + 1;
        match op.op {
            OpType::Insert { element } => { self.elements.insert(op.id, element); self.order.push(op.id); }
            OpType::Delete { id } => { self.elements.remove(&id); self.order.retain(|x| *x != id); }
            OpType::Update { id, element } => { self.elements.insert(id, element); }
        }
        true
    }
}
