pub mod class_diagram;
pub mod compartment;
pub mod edges;
pub mod entity_relationship;
pub mod flowchart;
pub mod graph;
pub mod node_shape;
pub mod route;
pub mod sequence;
pub mod state_diagram;
pub mod text;

use crate::scene::Scene;
use crate::schema::{Diagram, DiagramKind};
use crate::theme::Theme;

pub fn build_scene(
    diagram: &Diagram,
    theme: &Theme,
    measure: &mut dyn FnMut(&str, f32, bool) -> f32,
) -> Scene {
    match &diagram.kind {
        DiagramKind::Flowchart(data) => flowchart::generate(data, theme, measure),
        DiagramKind::Sequence(data) => sequence::generate(data, theme, measure),
        DiagramKind::Class(data) => class_diagram::generate(data, theme, measure),
        DiagramKind::State(data) => state_diagram::generate(data, theme, measure),
        DiagramKind::EntityRelationship(data) => {
            entity_relationship::generate(data, theme, measure)
        }
    }
}

pub fn approximate_measure(text: &str, font_size: f32, monospace: bool) -> f32 {
    if monospace {
        return text.chars().count() as f32 * font_size * 0.6;
    }
    text.chars()
        .map(|character| match character {
            'i' | 'l' | 'I' | 'j' | '.' | ',' | ':' | ';' | '\'' | '|' | '!' => 0.30,
            'f' | 't' | 'r' | '(' | ')' | '[' | ']' | '-' | ' ' => 0.36,
            'm' | 'M' | 'W' | 'w' | '@' => 0.92,
            'A'..='Z' => 0.68,
            '0'..='9' => 0.58,
            _ => 0.53,
        })
        .sum::<f32>()
        * font_size
}
