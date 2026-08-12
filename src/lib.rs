pub mod geometry;
pub mod layout;
pub mod measure;
pub mod render;
pub mod scene;
pub mod schema;
pub mod theme;

pub use layout::build_scene;
pub use schema::{Diagram, DiagramKind, parse, to_json};
pub use theme::{Theme, theme_by_name, theme_dark, theme_light};

pub fn scene_for(diagram: &Diagram, theme: &Theme) -> scene::Scene {
    let mut measurer = measure::new_measurer();
    build_scene(diagram, theme, &mut |text, size, monospace| {
        measure::measure_text(&mut measurer, text, size, monospace)
    })
}
