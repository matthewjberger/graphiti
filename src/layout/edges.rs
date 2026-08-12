use crate::geometry::trim_polyline_end;
use crate::layout::route::{
    ArrowStyle, BoxBounds, arrow_trim, box_center, build_path, entry_point, exit_point, opposite,
    push_arrow, stroke_edge,
};
use crate::scene::{Dash, LAYER_EDGE, Scene};
use crate::schema::{ArrowHead, Direction, EdgeRouting};
use crate::theme::{Rgba, Theme};
use nalgebra_glm::{Vec2, vec2};

#[derive(Clone, Copy, Debug)]
pub struct EdgeVisual {
    pub color: Rgba,
    pub width: f32,
    pub dash: Option<Dash>,
    pub head: ArrowHead,
    pub tail: ArrowHead,
}

pub struct EdgeRoute<'a> {
    pub waypoints: &'a [Vec2],
    pub source: BoxBounds,
    pub target: BoxBounds,
    pub direction: Direction,
    pub routing: EdgeRouting,
    pub lane_offset: f32,
}

pub fn lane_offsets(pairs: &[(usize, usize)], spacing: f32) -> Vec<f32> {
    let mut counts: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for &(from, to) in pairs {
        *counts.entry(unordered(from, to)).or_insert(0) += 1;
    }
    let mut seen: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    pairs
        .iter()
        .map(|&(from, to)| {
            let key = unordered(from, to);
            let total = counts[&key];
            if total < 2 {
                return 0.0;
            }
            let index = seen.entry(key).or_insert(0);
            let position = *index;
            *index += 1;
            (position as f32 - (total as f32 - 1.0) * 0.5) * spacing
        })
        .collect()
}

fn unordered(from: usize, to: usize) -> (usize, usize) {
    if from <= to { (from, to) } else { (to, from) }
}

pub fn draw_layered_edge(
    scene: &mut Scene,
    route: &EdgeRoute,
    visual: &EdgeVisual,
    theme: &Theme,
) -> Vec<Vec2> {
    let metrics = theme.metrics;
    let mut points: Vec<Vec2> = route.waypoints.to_vec();
    if points.len() < 2 {
        points = vec![box_center(route.source), box_center(route.target)];
    }

    let flow = if along_axis(box_center(route.target), route.direction)
        >= along_axis(box_center(route.source), route.direction)
    {
        route.direction
    } else {
        opposite(route.direction)
    };

    let cross_start = points.get(1).copied().unwrap_or(points[0]);
    let cross_end = points
        .get(points.len().saturating_sub(2))
        .copied()
        .unwrap_or(points[points.len() - 1]);
    let start = exit_point(
        route.source,
        flow,
        axis_cross(cross_start, flow) + route.lane_offset,
        0.0,
    );
    let end = entry_point(
        route.target,
        flow,
        axis_cross(cross_end, flow) + route.lane_offset,
        0.0,
    );

    let mut threaded = Vec::with_capacity(points.len());
    threaded.push(start);
    for point in points.iter().skip(1).take(points.len().saturating_sub(2)) {
        threaded.push(shift_cross(*point, flow, route.lane_offset));
    }
    threaded.push(end);

    let path = build_path(&threaded, route.routing, flow, metrics.corner_radius * 1.4);

    let forward = trim_polyline_end(&path, arrow_trim(visual.head, metrics.arrow_size));
    let reversed: Vec<Vec2> = forward.iter().rev().copied().collect();
    let trimmed = trim_polyline_end(&reversed, arrow_trim(visual.tail, metrics.arrow_size));
    let drawn: Vec<Vec2> = trimmed.iter().rev().copied().collect();

    stroke_edge(
        scene,
        &drawn,
        visual.width,
        visual.color,
        visual.dash,
        LAYER_EDGE,
    );

    let style = ArrowStyle {
        size: metrics.arrow_size,
        color: visual.color,
        background: theme.background,
        width: visual.width,
        depth: LAYER_EDGE,
    };
    if !matches!(visual.head, ArrowHead::None) && path.len() >= 2 {
        let tip = path[path.len() - 1];
        let previous = path[path.len() - 2];
        push_arrow(scene, visual.head, tip, tip - previous, style);
    }
    if !matches!(visual.tail, ArrowHead::None) && path.len() >= 2 {
        let tip = path[0];
        let next = path[1];
        push_arrow(scene, visual.tail, tip, tip - next, style);
    }

    path
}

fn shift_cross(point: Vec2, direction: Direction, offset: f32) -> Vec2 {
    match direction {
        Direction::Down | Direction::Up => vec2(point.x + offset, point.y),
        Direction::Right | Direction::Left => vec2(point.x, point.y + offset),
    }
}

fn axis_cross(point: Vec2, direction: Direction) -> f32 {
    match direction {
        Direction::Down | Direction::Up => point.x,
        Direction::Right | Direction::Left => point.y,
    }
}

fn along_axis(point: Vec2, direction: Direction) -> f32 {
    match direction {
        Direction::Down => point.y,
        Direction::Up => -point.y,
        Direction::Right => point.x,
        Direction::Left => -point.x,
    }
}
