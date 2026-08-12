use crate::theme::Rgba;
use nalgebra_glm::Vec2;

pub const LAYER_GROUP: f32 = 0.0;
pub const LAYER_GROUP_LABEL: f32 = 1.0;
pub const LAYER_EDGE: f32 = 2.0;
pub const LAYER_NODE: f32 = 3.0;
pub const LAYER_NODE_ACCENT: f32 = 4.0;
pub const LAYER_NODE_TEXT: f32 = 5.0;
pub const LAYER_EDGE_LABEL_BACKGROUND: f32 = 6.0;
pub const LAYER_EDGE_LABEL: f32 = 7.0;
pub const LAYER_TITLE: f32 = 8.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextBaseline {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug)]
pub struct Dash {
    pub on: f32,
    pub off: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Paint {
    pub fill: Option<Rgba>,
    pub stroke: Option<Rgba>,
    pub stroke_width: f32,
    pub dash: Option<Dash>,
    pub depth: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct LabelStyle {
    pub size: f32,
    pub color: Rgba,
    pub align: TextAlign,
    pub baseline: TextBaseline,
    pub depth: f32,
    pub monospace: bool,
}

pub fn paint_fill(color: Rgba, depth: f32) -> Paint {
    Paint {
        fill: Some(color),
        depth,
        ..Paint::default()
    }
}

pub fn paint_outline(color: Rgba, width: f32, depth: f32) -> Paint {
    Paint {
        stroke: Some(color),
        stroke_width: width,
        depth,
        ..Paint::default()
    }
}

pub fn paint_surface(fill: Rgba, stroke: Rgba, width: f32, depth: f32) -> Paint {
    Paint {
        fill: Some(fill),
        stroke: Some(stroke),
        stroke_width: width,
        depth,
        ..Paint::default()
    }
}

pub fn label_style(
    size: f32,
    color: Rgba,
    align: TextAlign,
    baseline: TextBaseline,
    depth: f32,
) -> LabelStyle {
    LabelStyle {
        size,
        color,
        align,
        baseline,
        depth,
        monospace: false,
    }
}

pub fn mono_label_style(style: LabelStyle) -> LabelStyle {
    LabelStyle {
        monospace: true,
        ..style
    }
}

#[derive(Clone, Debug)]
pub struct SceneRect {
    pub position: Vec2,
    pub size: Vec2,
    pub radius: f32,
    pub paint: Paint,
}

#[derive(Clone, Debug)]
pub struct ScenePolygon {
    pub points: Vec<Vec2>,
    pub paint: Paint,
}

#[derive(Clone, Debug)]
pub struct SceneCircle {
    pub center: Vec2,
    pub radius: f32,
    pub paint: Paint,
}

#[derive(Clone, Debug)]
pub struct SceneStroke {
    pub points: Vec<Vec2>,
    pub width: f32,
    pub color: Rgba,
    pub dash: Option<Dash>,
    pub depth: f32,
}

#[derive(Clone, Debug)]
pub struct SceneLabel {
    pub text: String,
    pub position: Vec2,
    pub style: LabelStyle,
}

#[derive(Clone, Debug, Default)]
pub struct Scene {
    pub size: Vec2,
    pub background: Option<Rgba>,
    pub rects: Vec<SceneRect>,
    pub polygons: Vec<ScenePolygon>,
    pub circles: Vec<SceneCircle>,
    pub strokes: Vec<SceneStroke>,
    pub labels: Vec<SceneLabel>,
}

pub fn push_rect(scene: &mut Scene, position: Vec2, size: Vec2, radius: f32, paint: Paint) {
    scene.rects.push(SceneRect {
        position,
        size,
        radius,
        paint,
    });
}

pub fn push_polygon(scene: &mut Scene, points: Vec<Vec2>, paint: Paint) {
    if points.len() < 3 {
        return;
    }
    scene.polygons.push(ScenePolygon { points, paint });
}

pub fn push_circle(scene: &mut Scene, center: Vec2, radius: f32, paint: Paint) {
    scene.circles.push(SceneCircle {
        center,
        radius,
        paint,
    });
}

pub fn push_stroke(
    scene: &mut Scene,
    points: Vec<Vec2>,
    width: f32,
    color: Rgba,
    dash: Option<Dash>,
    depth: f32,
) {
    if points.len() < 2 {
        return;
    }
    scene.strokes.push(SceneStroke {
        points,
        width,
        color,
        dash,
        depth,
    });
}

pub fn push_label(scene: &mut Scene, text: impl Into<String>, position: Vec2, style: LabelStyle) {
    let text = text.into();
    if text.is_empty() {
        return;
    }
    scene.labels.push(SceneLabel {
        text,
        position,
        style,
    });
}

pub fn dash_for_style(style: crate::schema::LineStyle, width: f32) -> Option<Dash> {
    match style {
        crate::schema::LineStyle::Solid | crate::schema::LineStyle::Thick => None,
        crate::schema::LineStyle::Dashed => Some(Dash {
            on: width * 4.0,
            off: width * 3.0,
        }),
        crate::schema::LineStyle::Dotted => Some(Dash {
            on: width,
            off: width * 2.0,
        }),
    }
}

pub fn width_for_style(style: crate::schema::LineStyle, base: f32) -> f32 {
    match style {
        crate::schema::LineStyle::Thick => base * 2.0,
        _ => base,
    }
}
