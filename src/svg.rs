use crate::scene::{Dash, Paint, Scene, TextAlign, TextBaseline};
use crate::theme::Rgba;
use nalgebra_glm::Vec2;
use std::fmt::Write;

const SANS_STACK: &str = "Roboto, 'Helvetica Neue', 'Segoe UI', system-ui, sans-serif";
const MONO_STACK: &str = "'JetBrains Mono', 'Cascadia Code', ui-monospace, monospace";

enum Element<'a> {
    Rect(&'a crate::scene::SceneRect),
    Polygon(&'a crate::scene::ScenePolygon),
    Circle(&'a crate::scene::SceneCircle),
    Stroke(&'a crate::scene::SceneStroke),
    Label(&'a crate::scene::SceneLabel),
}

fn depth_of(element: &Element) -> f32 {
    match element {
        Element::Rect(rect) => rect.paint.depth,
        Element::Polygon(polygon) => polygon.paint.depth,
        Element::Circle(circle) => circle.paint.depth,
        Element::Stroke(stroke) => stroke.depth,
        Element::Label(label) => label.style.depth,
    }
}

pub fn to_svg(scene: &Scene) -> String {
    let width = scene.size.x.max(1.0);
    let height = scene.size.y.max(1.0);
    let mut out = String::with_capacity(4096);

    let _ = write!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" \
viewBox=\"0 0 {} {}\" preserveAspectRatio=\"xMidYMid meet\" font-family=\"{}\">",
        round(width),
        round(height),
        round(width),
        round(height),
        SANS_STACK
    );

    if let Some(background) = scene.background {
        let _ = write!(
            out,
            "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
            round(width),
            round(height),
            color(background)
        );
    }

    let mut elements: Vec<Element> = Vec::with_capacity(
        scene.rects.len()
            + scene.polygons.len()
            + scene.circles.len()
            + scene.strokes.len()
            + scene.labels.len(),
    );
    elements.extend(scene.rects.iter().map(Element::Rect));
    elements.extend(scene.polygons.iter().map(Element::Polygon));
    elements.extend(scene.circles.iter().map(Element::Circle));
    elements.extend(scene.strokes.iter().map(Element::Stroke));
    elements.extend(scene.labels.iter().map(Element::Label));
    elements.sort_by(|left, right| {
        depth_of(left)
            .partial_cmp(&depth_of(right))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for element in &elements {
        match element {
            Element::Rect(rect) => {
                let _ = write!(
                    out,
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
                    round(rect.position.x),
                    round(rect.position.y),
                    round(rect.size.x),
                    round(rect.size.y)
                );
                if rect.radius > 0.01 {
                    let limit = rect.size.x.min(rect.size.y) * 0.5;
                    let _ = write!(out, " rx=\"{}\"", round(rect.radius.min(limit)));
                }
                write_paint(&mut out, &rect.paint);
                out.push_str("/>");
            }
            Element::Polygon(polygon) => {
                let _ = write!(out, "<polygon points=\"{}\"", points(&polygon.points));
                write_paint(&mut out, &polygon.paint);
                out.push_str("/>");
            }
            Element::Circle(circle) => {
                let _ = write!(
                    out,
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"",
                    round(circle.center.x),
                    round(circle.center.y),
                    round(circle.radius)
                );
                write_paint(&mut out, &circle.paint);
                out.push_str("/>");
            }
            Element::Stroke(stroke) => {
                let _ = write!(
                    out,
                    "<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" \
stroke-linecap=\"round\" stroke-linejoin=\"round\"",
                    points(&stroke.points),
                    color(stroke.color),
                    round(stroke.width)
                );
                write_dash(&mut out, stroke.dash);
                out.push_str("/>");
            }
            Element::Label(label) => {
                let anchor = match label.style.align {
                    TextAlign::Left => "start",
                    TextAlign::Center => "middle",
                    TextAlign::Right => "end",
                };
                let size = label.style.size;
                let (y, baseline) = match label.style.baseline {
                    TextBaseline::Middle => (label.position.y, " dominant-baseline=\"central\""),
                    TextBaseline::Top => (label.position.y + size * 0.8, ""),
                    TextBaseline::Bottom => (label.position.y - size * 0.2, ""),
                };
                let family = if label.style.monospace {
                    MONO_STACK
                } else {
                    SANS_STACK
                };
                let _ = write!(
                    out,
                    "<text x=\"{}\" y=\"{}\" font-size=\"{}\" font-family=\"{}\" fill=\"{}\" \
text-anchor=\"{}\"{}>{}</text>",
                    round(label.position.x),
                    round(y),
                    round(size),
                    family,
                    color(label.style.color),
                    anchor,
                    baseline,
                    escape(&label.text)
                );
            }
        }
    }

    out.push_str("</svg>");
    out
}

fn write_paint(out: &mut String, paint: &Paint) {
    match paint.fill {
        Some(fill) => {
            let _ = write!(out, " fill=\"{}\"", color(fill));
            if fill.alpha < 0.999 {
                let _ = write!(out, " fill-opacity=\"{}\"", round(fill.alpha));
            }
        }
        None => out.push_str(" fill=\"none\""),
    }
    if let Some(stroke) = paint.stroke
        && paint.stroke_width > 0.0
    {
        let _ = write!(
            out,
            " stroke=\"{}\" stroke-width=\"{}\"",
            color(stroke),
            round(paint.stroke_width)
        );
        write_dash(out, paint.dash);
    }
}

fn write_dash(out: &mut String, dash: Option<Dash>) {
    if let Some(dash) = dash {
        let _ = write!(
            out,
            " stroke-dasharray=\"{} {}\"",
            round(dash.on),
            round(dash.off)
        );
    }
}

fn points(values: &[Vec2]) -> String {
    let mut out = String::with_capacity(values.len() * 12);
    for (index, point) in values.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        let _ = write!(out, "{},{}", round(point.x), round(point.y));
    }
    out
}

fn linear_channel_to_srgb(value: f32) -> f32 {
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

pub fn color(value: Rgba) -> String {
    let channel =
        |component: f32| (linear_channel_to_srgb(component.clamp(0.0, 1.0)) * 255.0).round() as u8;
    format!(
        "#{:02X}{:02X}{:02X}",
        channel(value.red),
        channel(value.green),
        channel(value.blue)
    )
}

fn round(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded == rounded.trunc() {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded}")
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(character),
        }
    }
    out
}
