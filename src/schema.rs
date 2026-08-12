pub mod class_diagram;
pub mod common;
pub mod entity_relationship;
pub mod flowchart;
pub mod sequence;
pub mod state_diagram;

pub use class_diagram::ClassDiagram;
pub use common::{Accent, ArrowHead, Direction, EdgeRouting, LineStyle, NodeShape, Visibility};
pub use entity_relationship::EntityRelationship;
pub use flowchart::Flowchart;
pub use sequence::Sequence;
pub use state_diagram::StateDiagram;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diagram {
    pub kind: DiagramKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiagramKind {
    Flowchart(Flowchart),
    Sequence(Sequence),
    Class(ClassDiagram),
    State(StateDiagram),
    EntityRelationship(EntityRelationship),
}

pub fn parse(source: &str) -> Result<Diagram, serde_json::Error> {
    serde_json::from_str(source)
}

pub fn to_json(diagram: &Diagram) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(diagram)
}

pub fn kind_name(kind: &DiagramKind) -> &'static str {
    match kind {
        DiagramKind::Flowchart(_) => "flowchart",
        DiagramKind::Sequence(_) => "sequence",
        DiagramKind::Class(_) => "class",
        DiagramKind::State(_) => "state",
        DiagramKind::EntityRelationship(_) => "entity_relationship",
    }
}

pub fn title(kind: &DiagramKind) -> Option<&str> {
    match kind {
        DiagramKind::Flowchart(data) => data.title.as_deref(),
        DiagramKind::Sequence(data) => data.title.as_deref(),
        DiagramKind::Class(data) => data.title.as_deref(),
        DiagramKind::State(data) => data.title.as_deref(),
        DiagramKind::EntityRelationship(data) => data.title.as_deref(),
    }
}
