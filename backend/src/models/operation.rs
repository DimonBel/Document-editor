use serde::{Deserialize, Serialize};

/// All drawable element variants. `type` field in JSON discriminates the variant.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum ElementVariant {
    Freehand {
        points: Vec<f32>,
        color: String,
        width: f32,
    },
    Rectangle {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: String,
        fill: Option<String>,
    },
    Ellipse {
        x: f32,
        y: f32,
        rx: f32,
        ry: f32,
        color: String,
        fill: Option<String>,
    },
    Text {
        x: f32,
        y: f32,
        content: String,
        #[serde(rename = "fontSize")]
        font_size: f32,
        color: String,
    },
    Eraser {
        points: Vec<f32>,
        width: f32,
    },
}

/// A single canvas element carrying its own ID.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DrawElement {
    pub id: String,
    #[serde(flatten)]
    pub variant: ElementVariant,
}

/// The three mutation types for the CRDT log.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum OpType {
    Insert { element: DrawElement },
    Delete { id: String },
    Update { id: String, element: DrawElement },
}

/// A single CRDT operation with ordering metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: String,
    pub client_id: String,
    pub lamport: u64,
    pub op: OpType,
}
