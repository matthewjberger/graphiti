use crate::schema::common::{Accent, ArrowHead, Direction, EdgeRouting, LineStyle, NodeShape};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Flowchart {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub direction: Direction,
    #[serde(default)]
    pub routing: EdgeRouting,
    #[serde(default)]
    pub nodes: Vec<FlowNode>,
    #[serde(default)]
    pub edges: Vec<FlowEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<FlowGroup>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub shape: NodeShape,
    #[serde(default)]
    pub accent: Accent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub style: LineStyle,
    #[serde(default)]
    pub head: ArrowHead,
    #[serde(default = "no_arrow")]
    pub tail: ArrowHead,
    #[serde(default)]
    pub accent: Accent,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FlowGroup {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub nodes: Vec<String>,
    #[serde(default)]
    pub accent: Accent,
}

fn no_arrow() -> ArrowHead {
    ArrowHead::None
}
