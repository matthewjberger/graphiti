use crate::layout::text::Measure;
use crate::scene::{
    LAYER_NODE, LAYER_NODE_ACCENT, LAYER_NODE_TEXT, Scene, TextAlign, TextBaseline, label_style,
    paint_fill, paint_surface, push_label, push_rect, push_stroke,
};
use crate::theme::{Rgba, Theme};
use nalgebra_glm::{Vec2, vec2};

const PADDING: f32 = 14.0;

#[derive(Clone, Debug, Default)]
pub struct Row {
    pub text: String,
    pub badge: Option<String>,
    pub muted: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Compartment {
    pub rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct Content {
    pub header: Vec<String>,
    pub header_size: f32,
    pub subtitle: Option<String>,
    pub subtitle_size: f32,
    pub row_size: f32,
    pub compartments: Vec<Compartment>,
}

#[derive(Clone, Debug, Default)]
pub struct CompartmentLayout {
    pub size: Vec2,
    pub header_height: f32,
    pub compartment_heights: Vec<f32>,
    pub row_height: f32,
}

pub struct Style {
    pub fill: Rgba,
    pub header_fill: Rgba,
    pub border: Rgba,
    pub text: Rgba,
    pub muted: Rgba,
    pub accent: Rgba,
}

pub fn measure_compartments(
    content: &Content,
    theme: &Theme,
    measure: Measure,
) -> CompartmentLayout {
    let metrics = theme.metrics;
    let row_height = content.row_size * 1.75;

    let mut width = 0.0f32;
    for line in &content.header {
        width = width.max(measure(line, content.header_size));
    }
    if let Some(subtitle) = &content.subtitle {
        width = width.max(measure(subtitle, content.subtitle_size));
    }
    for compartment in &content.compartments {
        for row in &compartment.rows {
            let mut row_width = measure(&row.text, content.row_size);
            if let Some(badge) = &row.badge {
                row_width += measure(badge, content.row_size * 0.85) + 18.0;
            }
            width = width.max(row_width);
        }
    }
    width = (width + PADDING * 2.0).max(metrics.node_min_width);

    let subtitle_height = content
        .subtitle
        .as_ref()
        .map(|_| content.subtitle_size * 1.5)
        .unwrap_or(0.0);
    let header_height =
        content.header.len() as f32 * content.header_size * 1.5 + subtitle_height + PADDING * 0.9;
    let compartment_heights: Vec<f32> = content
        .compartments
        .iter()
        .map(|compartment| {
            if compartment.rows.is_empty() {
                0.0
            } else {
                compartment.rows.len() as f32 * row_height + PADDING * 0.6
            }
        })
        .collect();
    let total: f32 = compartment_heights.iter().sum();

    CompartmentLayout {
        size: vec2(width, header_height + total),
        header_height,
        compartment_heights,
        row_height,
    }
}

pub fn draw_compartments(
    scene: &mut Scene,
    layout: &CompartmentLayout,
    position: Vec2,
    content: &Content,
    style: &Style,
    theme: &Theme,
) {
    let metrics = theme.metrics;
    push_rect(
        scene,
        position,
        layout.size,
        metrics.corner_radius,
        paint_surface(style.fill, style.border, metrics.border_width, LAYER_NODE),
    );
    push_rect(
        scene,
        position,
        vec2(layout.size.x, layout.header_height),
        metrics.corner_radius,
        paint_fill(style.header_fill, LAYER_NODE + 0.01),
    );
    push_rect(
        scene,
        vec2(position.x, position.y + layout.header_height - 3.0),
        vec2(layout.size.x, 3.0),
        0.0,
        paint_fill(style.accent, LAYER_NODE_ACCENT),
    );

    let mut cursor = position.y + PADDING * 0.45;
    for line in &content.header {
        push_label(
            scene,
            line.clone(),
            vec2(
                position.x + layout.size.x * 0.5,
                cursor + content.header_size * 0.75,
            ),
            label_style(
                content.header_size,
                style.text,
                TextAlign::Center,
                TextBaseline::Middle,
                LAYER_NODE_TEXT,
            ),
        );
        cursor += content.header_size * 1.5;
    }
    if let Some(subtitle) = &content.subtitle {
        push_label(
            scene,
            subtitle.clone(),
            vec2(
                position.x + layout.size.x * 0.5,
                cursor + content.subtitle_size * 0.7,
            ),
            label_style(
                content.subtitle_size,
                style.muted,
                TextAlign::Center,
                TextBaseline::Middle,
                LAYER_NODE_TEXT,
            ),
        );
    }

    let mut section_top = position.y + layout.header_height;
    for (index, compartment) in content.compartments.iter().enumerate() {
        let height = layout.compartment_heights[index];
        if height <= 0.0 {
            continue;
        }
        push_stroke(
            scene,
            vec![
                vec2(position.x, section_top),
                vec2(position.x + layout.size.x, section_top),
            ],
            1.2,
            style.border,
            None,
            LAYER_NODE + 0.02,
        );
        let mut row_cursor = section_top + PADDING * 0.3;
        for row in &compartment.rows {
            let color = if row.muted { style.muted } else { style.text };
            push_label(
                scene,
                row.text.clone(),
                vec2(position.x + PADDING, row_cursor + layout.row_height * 0.5),
                label_style(
                    content.row_size,
                    color,
                    TextAlign::Left,
                    TextBaseline::Middle,
                    LAYER_NODE_TEXT,
                ),
            );
            if let Some(badge) = &row.badge {
                push_label(
                    scene,
                    badge.clone(),
                    vec2(
                        position.x + layout.size.x - PADDING,
                        row_cursor + layout.row_height * 0.5,
                    ),
                    label_style(
                        content.row_size * 0.85,
                        style.accent,
                        TextAlign::Right,
                        TextBaseline::Middle,
                        LAYER_NODE_TEXT,
                    ),
                );
            }
            row_cursor += layout.row_height;
        }
        section_top += height;
    }
}
