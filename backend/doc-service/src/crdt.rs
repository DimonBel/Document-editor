// TextDocument CRDT (rich-text). Ported from the legacy `backend/src/documents/crdt.rs` which was
// defined but never wired. Uses an op-based RGA-inspired structure.
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextOp { Insert { id: Uuid, after: Option<Uuid>, ch: char }, Delete { id: Uuid } }

#[derive(Default)]
pub struct TextDocument { pub chars: BTreeMap<Uuid, char>, pub order: Vec<Uuid>, pub rev: u64 }
impl TextDocument {
    pub fn apply(&mut self, op: TextOp) -> bool {
        self.rev += 1;
        match op {
            TextOp::Insert { id, after, ch } => {
                if self.chars.contains_key(&id) { return false; }
                self.chars.insert(id, ch);
                let pos = match after { Some(a) => self.order.iter().position(|x| *x == a).map(|p| p + 1).unwrap_or(self.order.len()), None => 0 };
                self.order.insert(pos, id); true
            }
            TextOp::Delete { id } => { self.chars.remove(&id); self.order.retain(|x| *x != id); true }
        }
    }
    pub fn to_string(&self) -> String { self.order.iter().filter_map(|i| self.chars.get(i)).collect() }
}
