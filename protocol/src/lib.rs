use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Render { source: String, theme: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Rendered {
        width: f32,
        height: f32,
        shapes: usize,
        labels: usize,
    },
    Failed {
        message: String,
    },
}
