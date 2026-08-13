pub mod geometry;
pub mod layout;
pub mod measure;
pub mod render;
pub mod scene;
pub mod schema;
pub mod svg;
pub mod theme;
pub mod validate;

pub use layout::build_scene;
pub use schema::{Diagram, DiagramKind, parse, to_json};
pub use svg::to_svg;
pub use theme::{Theme, theme_by_name, theme_dark, theme_light};
pub use validate::{Issue, Severity, issues};

pub fn scene_with(
    diagram: &Diagram,
    theme: &Theme,
    measurer: &mut measure::TextMeasurer,
) -> scene::Scene {
    build_scene(diagram, theme, &mut |text, size, monospace| {
        measure::measure_text(measurer, text, size, monospace)
    })
}

pub fn scene_for(diagram: &Diagram, theme: &Theme) -> scene::Scene {
    let mut measurer = measure::new_measurer();
    scene_with(diagram, theme, &mut measurer)
}
