use crate::scene::Dash;
use nalgebra_glm::{Vec2, vec2};

pub type Triangle = [Vec2; 3];

pub fn rounded_rect_outline(position: Vec2, size: Vec2, radius: f32, steps: usize) -> Vec<Vec2> {
    let limit = size.x.min(size.y) * 0.5;
    let radius = radius.min(limit).max(0.0);
    if radius <= 0.01 {
        return vec![
            position,
            vec2(position.x + size.x, position.y),
            vec2(position.x + size.x, position.y + size.y),
            vec2(position.x, position.y + size.y),
        ];
    }
    let steps = steps.max(2);
    let corners = [
        (
            vec2(position.x + size.x - radius, position.y + radius),
            -std::f32::consts::FRAC_PI_2,
        ),
        (
            vec2(position.x + size.x - radius, position.y + size.y - radius),
            0.0,
        ),
        (
            vec2(position.x + radius, position.y + size.y - radius),
            std::f32::consts::FRAC_PI_2,
        ),
        (
            vec2(position.x + radius, position.y + radius),
            std::f32::consts::PI,
        ),
    ];
    let mut points = Vec::with_capacity(steps * 4 + 4);
    for (center, start) in corners {
        for step in 0..=steps {
            let angle = start + std::f32::consts::FRAC_PI_2 * (step as f32 / steps as f32);
            points.push(vec2(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }
    }
    points
}

pub fn circle_outline(center: Vec2, radius: f32, segments: usize) -> Vec<Vec2> {
    let segments = segments.max(8);
    (0..segments)
        .map(|index| {
            let angle = std::f32::consts::TAU * index as f32 / segments as f32;
            vec2(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect()
}

pub fn polygon_signed_area(points: &[Vec2]) -> f32 {
    let mut total = 0.0;
    for index in 0..points.len() {
        let current = points[index];
        let next = points[(index + 1) % points.len()];
        total += current.x * next.y - next.x * current.y;
    }
    total * 0.5
}

pub fn triangulate_polygon(points: &[Vec2]) -> Vec<Triangle> {
    if points.len() < 3 {
        return Vec::new();
    }
    let mut working: Vec<Vec2> = points.to_vec();
    if polygon_signed_area(&working) < 0.0 {
        working.reverse();
    }
    let mut triangles = Vec::with_capacity(working.len());
    let mut guard = working.len() * working.len() + 16;
    while working.len() > 3 && guard > 0 {
        guard -= 1;
        let count = working.len();
        let mut clipped = false;
        for index in 0..count {
            let previous = working[(index + count - 1) % count];
            let current = working[index];
            let next = working[(index + 1) % count];
            if cross(current - previous, next - current) <= 0.0 {
                continue;
            }
            let contains_other = (0..count).any(|other| {
                other != index
                    && other != (index + count - 1) % count
                    && other != (index + 1) % count
                    && point_in_triangle(working[other], previous, current, next)
            });
            if contains_other {
                continue;
            }
            triangles.push([previous, current, next]);
            working.remove(index);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }
    if working.len() == 3 {
        triangles.push([working[0], working[1], working[2]]);
    }
    triangles
}

pub fn stroke_to_triangles(
    points: &[Vec2],
    width: f32,
    closed: bool,
    dash: Option<Dash>,
) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    let half = (width * 0.5).max(0.05);
    let paths = match dash {
        Some(dash) => split_dashes(points, closed, dash),
        None => {
            let mut path = points.to_vec();
            if closed && path.len() > 2 {
                path.push(points[0]);
            }
            vec![path]
        }
    };
    for path in paths {
        for window in path.windows(2) {
            let start = window[0];
            let end = window[1];
            let direction = end - start;
            let length = direction.norm();
            if length < 1.0e-5 {
                continue;
            }
            let normal = vec2(-direction.y / length, direction.x / length) * half;
            let a = start + normal;
            let b = end + normal;
            let c = end - normal;
            let d = start - normal;
            triangles.push([a, c, b]);
            triangles.push([a, d, c]);
        }
        let joint_count = path.len();
        for (index, point) in path.iter().enumerate() {
            let interior = index > 0 && index + 1 < joint_count;
            let endpoint = !interior;
            if interior || endpoint {
                triangles.extend(disc_triangles(*point, half, 10));
            }
        }
    }
    triangles
}

pub fn disc_triangles(center: Vec2, radius: f32, segments: usize) -> Vec<Triangle> {
    let outline = circle_outline(center, radius, segments);
    let mut triangles = Vec::with_capacity(outline.len());
    for index in 0..outline.len() {
        let next = (index + 1) % outline.len();
        triangles.push([center, outline[index], outline[next]]);
    }
    triangles
}

pub fn ring_triangles(center: Vec2, radius: f32, width: f32, segments: usize) -> Vec<Triangle> {
    let outer = circle_outline(center, radius + width * 0.5, segments);
    let inner = circle_outline(center, (radius - width * 0.5).max(0.01), segments);
    let mut triangles = Vec::with_capacity(segments * 2);
    for index in 0..outer.len() {
        let next = (index + 1) % outer.len();
        triangles.push([outer[index], outer[next], inner[next]]);
        triangles.push([outer[index], inner[next], inner[index]]);
    }
    triangles
}

pub fn polygon_outline_triangles(points: &[Vec2], width: f32) -> Vec<Triangle> {
    stroke_to_triangles(points, width, true, None)
}

pub fn polyline_length(points: &[Vec2]) -> f32 {
    points
        .windows(2)
        .map(|window| (window[1] - window[0]).norm())
        .sum()
}

pub fn point_at_distance(points: &[Vec2], distance: f32) -> (Vec2, Vec2) {
    let mut remaining = distance;
    for window in points.windows(2) {
        let start = window[0];
        let end = window[1];
        let segment = end - start;
        let length = segment.norm();
        if length < 1.0e-5 {
            continue;
        }
        if remaining <= length {
            let direction = segment / length;
            return (start + direction * remaining, direction);
        }
        remaining -= length;
    }
    let last = points.len() - 1;
    let direction = if points.len() >= 2 {
        let segment = points[last] - points[last - 1];
        let length = segment.norm();
        if length < 1.0e-5 {
            vec2(1.0, 0.0)
        } else {
            segment / length
        }
    } else {
        vec2(1.0, 0.0)
    };
    (points[last], direction)
}

pub fn trim_polyline_end(points: &[Vec2], amount: f32) -> Vec<Vec2> {
    if points.len() < 2 || amount <= 0.0 {
        return points.to_vec();
    }
    let total = polyline_length(points);
    if amount >= total {
        return points.to_vec();
    }
    let target = total - amount;
    let mut result = Vec::with_capacity(points.len());
    let mut travelled = 0.0;
    result.push(points[0]);
    for window in points.windows(2) {
        let start = window[0];
        let end = window[1];
        let segment = end - start;
        let length = segment.norm();
        if length < 1.0e-5 {
            continue;
        }
        if travelled + length >= target {
            let direction = segment / length;
            result.push(start + direction * (target - travelled));
            break;
        }
        travelled += length;
        result.push(end);
    }
    result
}

fn split_dashes(points: &[Vec2], closed: bool, dash: Dash) -> Vec<Vec<Vec2>> {
    let mut path = points.to_vec();
    if closed && path.len() > 2 {
        path.push(points[0]);
    }
    let period = (dash.on + dash.off).max(0.01);
    let total = polyline_length(&path);
    let mut result = Vec::new();
    let mut start = 0.0;
    while start < total {
        let end = (start + dash.on).min(total);
        result.push(sub_polyline(&path, start, end));
        start += period;
    }
    result.retain(|segment| segment.len() >= 2);
    result
}

fn sub_polyline(points: &[Vec2], from: f32, to: f32) -> Vec<Vec2> {
    let mut result = Vec::new();
    let mut travelled = 0.0;
    for window in points.windows(2) {
        let start = window[0];
        let end = window[1];
        let segment = end - start;
        let length = segment.norm();
        if length < 1.0e-5 {
            continue;
        }
        let direction = segment / length;
        let segment_start = travelled;
        let segment_end = travelled + length;
        if segment_end >= from && segment_start <= to {
            let enter = from.max(segment_start) - segment_start;
            let exit = to.min(segment_end) - segment_start;
            if exit > enter {
                let first = start + direction * enter;
                let last = start + direction * exit;
                if result.is_empty() {
                    result.push(first);
                }
                result.push(last);
            }
        }
        travelled = segment_end;
    }
    result
}

fn cross(left: Vec2, right: Vec2) -> f32 {
    left.x * right.y - left.y * right.x
}

fn point_in_triangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let first = cross(b - a, point - a);
    let second = cross(c - b, point - b);
    let third = cross(a - c, point - c);
    (first >= 0.0 && second >= 0.0 && third >= 0.0)
        || (first <= 0.0 && second <= 0.0 && third <= 0.0)
}
