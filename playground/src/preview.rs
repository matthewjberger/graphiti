use graphiti::scene::Scene;
use graphiti::schema::Diagram;
use graphiti::{measure, scene_with, schema, theme, to_svg};
use std::cell::RefCell;

thread_local! {
    static MEASURER: RefCell<measure::TextMeasurer> = RefCell::new(measure::new_measurer());
}

#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    pub svg: String,
    pub kind: &'static str,
    pub width: f32,
    pub height: f32,
    pub shapes: usize,
    pub labels: usize,
}

pub fn preview(diagram: &Diagram, theme_name: &str) -> Preview {
    let base = theme::theme_by_name(theme_name).unwrap_or_else(theme::theme_light);
    let scene = MEASURER.with_borrow_mut(|measurer| scene_with(diagram, &base, measurer));
    Preview {
        svg: to_svg(&scene),
        kind: schema::kind_name(&diagram.kind),
        width: scene.size.x,
        height: scene.size.y,
        shapes: shape_count(&scene),
        labels: scene.labels.len(),
    }
}

fn shape_count(scene: &Scene) -> usize {
    scene.rects.len() + scene.polygons.len() + scene.circles.len() + scene.strokes.len()
}
