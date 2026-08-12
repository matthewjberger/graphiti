use crate::geometry::{point_at_distance, polyline_length};
use crate::scene::{
    Dash, LAYER_EDGE_LABEL, LAYER_EDGE_LABEL_BACKGROUND, Scene, TextAlign, TextBaseline,
    label_style, paint_fill, paint_surface, push_circle, push_label, push_polygon, push_rect,
    push_stroke,
};
use crate::schema::{ArrowHead, Direction, EdgeRouting};
use crate::theme::Rgba;
use nalgebra_glm::{Vec2, vec2};

#[derive(Clone, Copy, Debug)]
pub struct BoxBounds {
    pub position: Vec2,
    pub size: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct ArrowStyle {
    pub size: f32,
    pub color: Rgba,
    pub background: Rgba,
    pub width: f32,
    pub depth: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeLabelStyle {
    pub size: f32,
    pub color: Rgba,
    pub background: Rgba,
    pub lateral: f32,
}

pub fn box_center(bounds: BoxBounds) -> Vec2 {
    vec2(
        bounds.position.x + bounds.size.x * 0.5,
        bounds.position.y + bounds.size.y * 0.5,
    )
}

pub fn exit_point(bounds: BoxBounds, direction: Direction, cross: f32, inset: f32) -> Vec2 {
    let center = box_center(bounds);
    match direction {
        Direction::Down => vec2(
            clamp_cross(cross, center.x, bounds.size.x),
            bounds.position.y + bounds.size.y + inset,
        ),
        Direction::Up => vec2(
            clamp_cross(cross, center.x, bounds.size.x),
            bounds.position.y - inset,
        ),
        Direction::Right => vec2(
            bounds.position.x + bounds.size.x + inset,
            clamp_cross(cross, center.y, bounds.size.y),
        ),
        Direction::Left => vec2(
            bounds.position.x - inset,
            clamp_cross(cross, center.y, bounds.size.y),
        ),
    }
}

pub fn entry_point(bounds: BoxBounds, direction: Direction, cross: f32, inset: f32) -> Vec2 {
    exit_point(bounds, opposite(direction), cross, inset)
}

pub fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::Down => Direction::Up,
        Direction::Up => Direction::Down,
        Direction::Right => Direction::Left,
        Direction::Left => Direction::Right,
    }
}

fn clamp_cross(value: f32, center: f32, extent: f32) -> f32 {
    let limit = (extent * 0.5 - 10.0).max(0.0);
    value.clamp(center - limit, center + limit)
}

pub fn build_path(
    waypoints: &[Vec2],
    routing: EdgeRouting,
    direction: Direction,
    corner_radius: f32,
) -> Vec<Vec2> {
    if waypoints.len() < 2 {
        return waypoints.to_vec();
    }
    match routing {
        EdgeRouting::Straight => waypoints.to_vec(),
        EdgeRouting::Curved => sample_spline(waypoints, 12),
        EdgeRouting::Orthogonal => {
            let staircase = orthogonal_path(waypoints, direction);
            round_corners(&staircase, corner_radius)
        }
    }
}

fn orthogonal_path(waypoints: &[Vec2], direction: Direction) -> Vec<Vec2> {
    let vertical = matches!(direction, Direction::Down | Direction::Up);
    let mut result = vec![waypoints[0]];
    for window in waypoints.windows(2) {
        let from = window[0];
        let to = window[1];
        if vertical {
            if (from.x - to.x).abs() > 0.5 {
                let middle = (from.y + to.y) * 0.5;
                result.push(vec2(from.x, middle));
                result.push(vec2(to.x, middle));
            }
        } else if (from.y - to.y).abs() > 0.5 {
            let middle = (from.x + to.x) * 0.5;
            result.push(vec2(middle, from.y));
            result.push(vec2(middle, to.y));
        }
        result.push(to);
    }
    dedupe(result)
}

fn dedupe(points: Vec<Vec2>) -> Vec<Vec2> {
    let mut result: Vec<Vec2> = Vec::with_capacity(points.len());
    for point in points {
        if result
            .last()
            .map(|last| (last - point).norm() > 0.5)
            .unwrap_or(true)
        {
            result.push(point);
        }
    }
    result
}

pub fn round_corners(points: &[Vec2], radius: f32) -> Vec<Vec2> {
    if points.len() < 3 || radius <= 0.1 {
        return points.to_vec();
    }
    let mut result = vec![points[0]];
    for index in 1..points.len() - 1 {
        let previous = points[index - 1];
        let current = points[index];
        let next = points[index + 1];
        let incoming = current - previous;
        let outgoing = next - current;
        let incoming_length = incoming.norm();
        let outgoing_length = outgoing.norm();
        if incoming_length < 1.0e-4 || outgoing_length < 1.0e-4 {
            continue;
        }
        let limit = radius.min(incoming_length * 0.5).min(outgoing_length * 0.5);
        let start = current - incoming / incoming_length * limit;
        let end = current + outgoing / outgoing_length * limit;
        result.push(start);
        for step in 1..4 {
            let t = step as f32 / 4.0;
            let one_minus = 1.0 - t;
            result.push(
                start * (one_minus * one_minus) + current * (2.0 * one_minus * t) + end * (t * t),
            );
        }
        result.push(end);
    }
    result.push(points[points.len() - 1]);
    dedupe(result)
}

fn sample_spline(points: &[Vec2], steps_per_segment: usize) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut extended = Vec::with_capacity(points.len() + 2);
    extended.push(points[0] + (points[0] - points[1]));
    extended.extend_from_slice(points);
    let last = points[points.len() - 1];
    extended.push(last + (last - points[points.len() - 2]));

    let mut result = Vec::new();
    for index in 1..extended.len() - 2 {
        let p0 = extended[index - 1];
        let p1 = extended[index];
        let p2 = extended[index + 1];
        let p3 = extended[index + 2];
        for step in 0..steps_per_segment {
            let t = step as f32 / steps_per_segment as f32;
            result.push(catmull_rom(p0, p1, p2, p3, t));
        }
    }
    result.push(last);
    dedupe(result)
}

fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    (p1 * 2.0
        + (p2 - p0) * t
        + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t2
        + (p1 * 3.0 - p0 - p2 * 3.0 + p3) * t3)
        * 0.5
}

pub fn arrow_trim(kind: ArrowHead, size: f32) -> f32 {
    match kind {
        ArrowHead::None | ArrowHead::Open | ArrowHead::Bar => 0.0,
        ArrowHead::Arrow => size * 0.85,
        ArrowHead::HollowTriangle => size,
        ArrowHead::Diamond | ArrowHead::HollowDiamond => size * 1.6,
        ArrowHead::Circle | ArrowHead::HollowCircle => size * 0.8,
        ArrowHead::CrowsFoot
        | ArrowHead::CrowsFootOne
        | ArrowHead::CrowsFootZeroOrOne
        | ArrowHead::CrowsFootZeroOrMany
        | ArrowHead::CrowsFootOneOrMany => 0.0,
    }
}

pub fn push_arrow(
    scene: &mut Scene,
    kind: ArrowHead,
    tip: Vec2,
    direction: Vec2,
    style: ArrowStyle,
) {
    let length = direction.norm();
    if length < 1.0e-4 {
        return;
    }
    let forward = direction / length;
    let side = vec2(-forward.y, forward.x);
    let size = style.size;
    match kind {
        ArrowHead::None => {}
        ArrowHead::Arrow => {
            let base = tip - forward * size;
            push_polygon(
                scene,
                vec![tip, base + side * size * 0.42, base - side * size * 0.42],
                paint_fill(style.color, style.depth),
            );
        }
        ArrowHead::Open => {
            let base = tip - forward * size;
            push_stroke(
                scene,
                vec![base + side * size * 0.55, tip, base - side * size * 0.55],
                style.width,
                style.color,
                None,
                style.depth,
            );
        }
        ArrowHead::HollowTriangle => {
            let base = tip - forward * size;
            push_polygon(
                scene,
                vec![tip, base + side * size * 0.5, base - side * size * 0.5],
                paint_surface(style.background, style.color, style.width, style.depth),
            );
        }
        ArrowHead::Diamond | ArrowHead::HollowDiamond => {
            let long = size * 1.6;
            let middle = tip - forward * long * 0.5;
            let back = tip - forward * long;
            let fill = if matches!(kind, ArrowHead::Diamond) {
                style.color
            } else {
                style.background
            };
            push_polygon(
                scene,
                vec![
                    tip,
                    middle + side * size * 0.45,
                    back,
                    middle - side * size * 0.45,
                ],
                paint_surface(fill, style.color, style.width, style.depth),
            );
        }
        ArrowHead::Circle | ArrowHead::HollowCircle => {
            let radius = size * 0.4;
            let center = tip - forward * radius;
            let fill = if matches!(kind, ArrowHead::Circle) {
                style.color
            } else {
                style.background
            };
            push_circle(
                scene,
                center,
                radius,
                paint_surface(fill, style.color, style.width, style.depth),
            );
        }
        ArrowHead::Bar => {
            let half = size * 0.5;
            push_stroke(
                scene,
                vec![tip + side * half, tip - side * half],
                style.width,
                style.color,
                None,
                style.depth,
            );
        }
        ArrowHead::CrowsFoot
        | ArrowHead::CrowsFootOne
        | ArrowHead::CrowsFootZeroOrOne
        | ArrowHead::CrowsFootZeroOrMany
        | ArrowHead::CrowsFootOneOrMany => {
            push_crows_foot(scene, kind, tip, forward, side, style);
        }
    }
}

fn push_crows_foot(
    scene: &mut Scene,
    kind: ArrowHead,
    tip: Vec2,
    forward: Vec2,
    side: Vec2,
    style: ArrowStyle,
) {
    let size = style.size;
    let many = matches!(
        kind,
        ArrowHead::CrowsFoot | ArrowHead::CrowsFootZeroOrMany | ArrowHead::CrowsFootOneOrMany
    );
    let optional = matches!(
        kind,
        ArrowHead::CrowsFootZeroOrOne | ArrowHead::CrowsFootZeroOrMany
    );
    let mandatory = matches!(
        kind,
        ArrowHead::CrowsFootOne | ArrowHead::CrowsFootOneOrMany
    );
    let span = size * 0.55;
    if many {
        let base = tip - forward * size;
        for target in [base + side * span, base - side * span, base] {
            push_stroke(
                scene,
                vec![tip, target],
                style.width,
                style.color,
                None,
                style.depth,
            );
        }
    }
    let bar_offset = if many { size * 1.15 } else { size * 0.5 };
    if mandatory || many {
        let center = tip - forward * bar_offset;
        push_stroke(
            scene,
            vec![center + side * span, center - side * span],
            style.width,
            style.color,
            None,
            style.depth,
        );
    }
    if optional {
        let radius = size * 0.34;
        let center = tip - forward * (bar_offset + radius * 1.6);
        push_circle(
            scene,
            center,
            radius,
            paint_surface(style.background, style.color, style.width, style.depth),
        );
    }
}

pub fn push_edge_label(
    scene: &mut Scene,
    text: &str,
    path: &[Vec2],
    style: EdgeLabelStyle,
    measure: &mut dyn FnMut(&str, f32, bool) -> f32,
) {
    if text.is_empty() || path.len() < 2 {
        return;
    }
    let anchor = label_anchor(path) + label_lateral(path, style.lateral);
    let width = measure(text, style.size, false);
    let padding = 5.0;
    push_rect(
        scene,
        vec2(
            anchor.x - width * 0.5 - padding,
            anchor.y - style.size * 0.5 - padding * 0.6,
        ),
        vec2(width + padding * 2.0, style.size + padding * 1.2),
        4.0,
        paint_fill(style.background, LAYER_EDGE_LABEL_BACKGROUND),
    );
    push_label(
        scene,
        text,
        anchor,
        label_style(
            style.size,
            style.color,
            TextAlign::Center,
            TextBaseline::Middle,
            LAYER_EDGE_LABEL,
        ),
    );
}

fn label_lateral(path: &[Vec2], amount: f32) -> Vec2 {
    if amount.abs() < 0.01 {
        return vec2(0.0, 0.0);
    }
    let mut best_length = 0.0;
    let mut direction = vec2(1.0, 0.0);
    for window in path.windows(2) {
        let segment = window[1] - window[0];
        let length = segment.norm();
        if length > best_length {
            best_length = length;
            direction = segment / length;
        }
    }
    vec2(-direction.y, direction.x) * amount
}

fn label_anchor(path: &[Vec2]) -> Vec2 {
    let total = polyline_length(path);
    let mut best_length = 0.0;
    let mut best_anchor = point_at_distance(path, total * 0.5).0;
    for window in path.windows(2) {
        let length = (window[1] - window[0]).norm();
        if length > best_length {
            best_length = length;
            best_anchor = (window[0] + window[1]) * 0.5;
        }
    }
    if best_length < 26.0 {
        return point_at_distance(path, total * 0.5).0;
    }
    best_anchor
}

pub fn push_end_label(
    scene: &mut Scene,
    text: &str,
    path: &[Vec2],
    from_start: bool,
    size: f32,
    color: Rgba,
) {
    if text.is_empty() || path.len() < 2 {
        return;
    }
    let total = polyline_length(path);
    let distance = if from_start {
        (total * 0.18).min(30.0)
    } else {
        total - (total * 0.18).min(30.0)
    };
    let (anchor, direction) = point_at_distance(path, distance);
    let side = vec2(-direction.y, direction.x);
    push_label(
        scene,
        text,
        anchor + side * 11.0,
        label_style(
            size,
            color,
            TextAlign::Center,
            TextBaseline::Middle,
            LAYER_EDGE_LABEL,
        ),
    );
}

pub fn stroke_edge(
    scene: &mut Scene,
    path: &[Vec2],
    width: f32,
    color: Rgba,
    dash: Option<Dash>,
    depth: f32,
) {
    push_stroke(scene, path.to_vec(), width, color, dash, depth);
}
