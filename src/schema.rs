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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagram {
    pub kind: DiagramKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

pub fn kind_names() -> &'static [&'static str] {
    &[
        "flowchart",
        "sequence",
        "class",
        "state",
        "entity_relationship",
    ]
}

pub fn kind_from_name(name: &str) -> Option<DiagramKind> {
    match name {
        "flowchart" => Some(DiagramKind::Flowchart(Flowchart::default())),
        "sequence" => Some(DiagramKind::Sequence(Sequence::default())),
        "class" => Some(DiagramKind::Class(ClassDiagram::default())),
        "state" => Some(DiagramKind::State(StateDiagram::default())),
        "entity_relationship" => Some(DiagramKind::EntityRelationship(
            EntityRelationship::default(),
        )),
        _ => None,
    }
}

pub fn style(kind: &DiagramKind) -> &common::Style {
    match kind {
        DiagramKind::Flowchart(data) => &data.style,
        DiagramKind::Sequence(data) => &data.style,
        DiagramKind::Class(data) => &data.style,
        DiagramKind::State(data) => &data.style,
        DiagramKind::EntityRelationship(data) => &data.style,
    }
}

pub fn style_mut(kind: &mut DiagramKind) -> &mut common::Style {
    match kind {
        DiagramKind::Flowchart(data) => &mut data.style,
        DiagramKind::Sequence(data) => &mut data.style,
        DiagramKind::Class(data) => &mut data.style,
        DiagramKind::State(data) => &mut data.style,
        DiagramKind::EntityRelationship(data) => &mut data.style,
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

pub fn title_mut(kind: &mut DiagramKind) -> &mut Option<String> {
    match kind {
        DiagramKind::Flowchart(data) => &mut data.title,
        DiagramKind::Sequence(data) => &mut data.title,
        DiagramKind::Class(data) => &mut data.title,
        DiagramKind::State(data) => &mut data.title,
        DiagramKind::EntityRelationship(data) => &mut data.title,
    }
}
