use crate::scene::{Paint, Scene, paint_outline, push_polygon, push_rect, push_stroke};
use crate::schema::NodeShape;
use crate::theme::Metrics;
use nalgebra_glm::{Vec2, vec2};

pub fn shape_size(shape: NodeShape, content: Vec2, metrics: &Metrics) -> Vec2 {
    let base = vec2(
        (content.x + metrics.node_padding_x * 2.0).max(metrics.node_min_width),
        (content.y + metrics.node_padding_y * 2.0).max(metrics.node_min_height),
    );
    match shape {
        NodeShape::Rectangle | NodeShape::Rounded | NodeShape::Note => base,
        NodeShape::Stadium => vec2(base.x + base.y * 0.35, base.y),
        NodeShape::Subroutine => vec2(base.x + 22.0, base.y),
        NodeShape::Parallelogram => vec2(base.x + base.y * 0.5, base.y),
        NodeShape::Hexagon => vec2(base.x + base.y * 0.7, base.y),
        NodeShape::Cylinder => vec2(base.x, base.y + 20.0),
        NodeShape::Diamond => vec2(
            (content.x * 1.7 + metrics.node_padding_x * 2.0).max(metrics.node_min_width * 1.5),
            (content.y * 2.0 + metrics.node_padding_y * 2.0).max(metrics.node_min_height * 1.3),
        ),
        NodeShape::Circle => {
            let diameter = base.x.max(base.y * 1.35) * 1.02;
            vec2(diameter, diameter)
        }
    }
}

pub fn push_node_shape(
    scene: &mut Scene,
    shape: NodeShape,
    position: Vec2,
    size: Vec2,
    paint: Paint,
    corner_radius: f32,
) {
    let border = paint.stroke.unwrap_or_default();
    let detail = paint_outline(border, paint.stroke_width, paint.depth + 0.01);
    match shape {
        NodeShape::Rectangle => push_rect(scene, position, size, 0.0, paint),
        NodeShape::Rounded => push_rect(scene, position, size, corner_radius, paint),
        NodeShape::Stadium => push_rect(scene, position, size, size.y * 0.5, paint),
        NodeShape::Circle => crate::scene::push_circle(
            scene,
            vec2(position.x + size.x * 0.5, position.y + size.y * 0.5),
            size.x.min(size.y) * 0.5,
            paint,
        ),
        NodeShape::Diamond => push_polygon(
            scene,
            vec![
                vec2(position.x + size.x * 0.5, position.y),
                vec2(position.x + size.x, position.y + size.y * 0.5),
                vec2(position.x + size.x * 0.5, position.y + size.y),
                vec2(position.x, position.y + size.y * 0.5),
            ],
            paint,
        ),
        NodeShape::Hexagon => {
            let notch = (size.y * 0.35).min(size.x * 0.3);
            push_polygon(
                scene,
                vec![
                    vec2(position.x + notch, position.y),
                    vec2(position.x + size.x - notch, position.y),
                    vec2(position.x + size.x, position.y + size.y * 0.5),
                    vec2(position.x + size.x - notch, position.y + size.y),
                    vec2(position.x + notch, position.y + size.y),
                    vec2(position.x, position.y + size.y * 0.5),
                ],
                paint,
            )
        }
        NodeShape::Parallelogram => {
            let skew = (size.y * 0.4).min(size.x * 0.35);
            push_polygon(
                scene,
                vec![
                    vec2(position.x + skew, position.y),
                    vec2(position.x + size.x, position.y),
                    vec2(position.x + size.x - skew, position.y + size.y),
                    vec2(position.x, position.y + size.y),
                ],
                paint,
            )
        }
        NodeShape::Cylinder => {
            let lip = (size.y * 0.16).min(14.0);
            let steps = 14;
            let mut points = Vec::new();
            for step in 0..=steps {
                let angle = std::f32::consts::PI * step as f32 / steps as f32;
                points.push(vec2(
                    position.x + size.x * 0.5 - (size.x * 0.5) * angle.cos(),
                    position.y + lip - lip * angle.sin(),
                ));
            }
            for step in 0..=steps {
                let angle = std::f32::consts::PI * step as f32 / steps as f32;
                points.push(vec2(
                    position.x + size.x * 0.5 + (size.x * 0.5) * angle.cos(),
                    position.y + size.y - lip + lip * angle.sin(),
                ));
            }
            push_polygon(scene, points, paint);
            let mut cap = Vec::new();
            for step in 0..=steps {
                let angle = std::f32::consts::PI * step as f32 / steps as f32;
                cap.push(vec2(
                    position.x + size.x * 0.5 - (size.x * 0.5) * angle.cos(),
                    position.y + lip + lip * angle.sin(),
                ));
            }
            push_stroke(scene, cap, detail.stroke_width, border, None, detail.depth);
        }
        NodeShape::Subroutine => {
            push_rect(scene, position, size, 0.0, paint);
            let inset = 11.0;
            push_stroke(
                scene,
                vec![
                    vec2(position.x + inset, position.y),
                    vec2(position.x + inset, position.y + size.y),
                ],
                detail.stroke_width,
                border,
                None,
                detail.depth,
            );
            push_stroke(
                scene,
                vec![
                    vec2(position.x + size.x - inset, position.y),
                    vec2(position.x + size.x - inset, position.y + size.y),
                ],
                detail.stroke_width,
                border,
                None,
                detail.depth,
            );
        }
        NodeShape::Note => {
            let fold = 16.0f32.min(size.x * 0.3).min(size.y * 0.4);
            push_polygon(
                scene,
                vec![
                    position,
                    vec2(position.x + size.x - fold, position.y),
                    vec2(position.x + size.x, position.y + fold),
                    vec2(position.x + size.x, position.y + size.y),
                    vec2(position.x, position.y + size.y),
                ],
                paint,
            );
            push_stroke(
                scene,
                vec![
                    vec2(position.x + size.x - fold, position.y),
                    vec2(position.x + size.x - fold, position.y + fold),
                    vec2(position.x + size.x, position.y + fold),
                ],
                detail.stroke_width,
                border,
                None,
                detail.depth,
            );
        }
    }
}
