use crate::schema::common::{Accent, Direction, LineStyle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StateDiagram {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub direction: Direction,
    #[serde(default)]
    pub states: Vec<State>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub kind: StateKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<String>,
    #[serde(default)]
    pub accent: Accent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateKind {
    #[default]
    Simple,
    Start,
    End,
    Choice,
    Fork,
    Join,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub style: LineStyle,
}
