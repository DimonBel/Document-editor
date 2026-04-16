use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextOp {
    pub id: String,
    pub client_id: String,
    pub lamport: u64,
    #[serde(rename = "type")]
    pub op_type: String,
    pub position: usize,
    pub text: Option<String>,
    pub length: Option<usize>,
}

pub struct TextDocument {
    content: String,
    version: u64,
    operations: VecDeque<TextOp>,
    lamport_clock: u64,
}

impl TextDocument {
    pub fn new() -> Self {
        TextDocument {
            content: String::new(),
            version: 0,
            operations: VecDeque::new(),
            lamport_clock: 0,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn insert(&mut self, position: usize, text: &str) -> TextOp {
        self.lamport_clock += 1;
        let op = TextOp {
            id: uuid::Uuid::new_v4().to_string(),
            client_id: String::new(),
            lamport: self.lamport_clock,
            op_type: "insert".to_string(),
            position,
            text: Some(text.to_string()),
            length: None,
        };
        self.apply_op(op.clone());
        op
    }

    pub fn delete(&mut self, position: usize, length: usize) -> TextOp {
        self.lamport_clock += 1;
        let op = TextOp {
            id: uuid::Uuid::new_v4().to_string(),
            client_id: String::new(),
            lamport: self.lamport_clock,
            op_type: "delete".to_string(),
            position,
            text: None,
            length: Some(length),
        };
        self.apply_op(op.clone());
        op
    }

    pub fn apply(&mut self, op: TextOp) {
        self.apply_op(op);
    }

    fn apply_op(&mut self, op: TextOp) {
        self.lamport_clock = self.lamport_clock.max(op.lamport) + 1;

        if self.operations.iter().any(|o| o.id == op.id) {
            return;
        }

        match op.op_type.as_str() {
            "insert" => {
                if let Some(ref text) = op.text {
                    let pos = op.position.min(self.content.len());
                    self.content.insert_str(pos, text);
                }
            }
            "delete" => {
                if let Some(len) = op.length {
                    let start = op.position.min(self.content.len());
                    let end = (start + len).min(self.content.len());
                    self.content.drain(start..end);
                }
            }
            _ => {}
        }

        self.operations.push_back(op);
        self.version += 1;

        while self.operations.len() > 1000 {
            self.operations.pop_front();
        }
    }
}
