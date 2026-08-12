use crate::layout::edges::{EdgeRoute, EdgeVisual, draw_layered_edge, lane_offsets};
use crate::layout::graph::{GraphInput, layout_layered};
use crate::layout::route::{BoxBounds, EdgeLabelStyle, box_center, push_edge_label};
use crate::layout::text::{Measure, TextBlock, measure_block};
use crate::scene::{
    LAYER_NODE, LAYER_NODE_TEXT, LAYER_TITLE, Scene, TextAlign, TextBaseline, dash_for_style,
    label_style, paint_fill, paint_surface, push_circle, push_label, push_polygon, push_rect,
    push_stroke, width_for_style,
};
use crate::schema::ArrowHead;
use crate::schema::state_diagram::{State, StateDiagram, StateKind};
use crate::theme::{Theme, accent_colors};
use nalgebra_glm::{Vec2, vec2};

const MARKER_RADIUS: f32 = 13.0;
const BAR_THICKNESS: f32 = 9.0;
const BAR_LENGTH: f32 = 74.0;

pub fn generate(data: &StateDiagram, theme: &Theme, measure: Measure) -> Scene {
    let metrics = theme.metrics;
    let mut scene = Scene {
        background: Some(theme.background),
        ..Scene::default()
    };

    let vertical = matches!(
        data.direction,
        crate::schema::Direction::Down | crate::schema::Direction::Up
    );

    let mut blocks: Vec<Option<TextBlock>> = Vec::with_capacity(data.states.len());
    let mut sizes = Vec::with_capacity(data.states.len());
    for state in &data.states {
        match state.kind {
            StateKind::Start | StateKind::End => {
                blocks.push(None);
                sizes.push(vec2(MARKER_RADIUS * 2.0, MARKER_RADIUS * 2.0));
            }
            StateKind::Choice => {
                blocks.push(None);
                sizes.push(vec2(46.0, 46.0));
            }
            StateKind::Fork | StateKind::Join => {
                blocks.push(None);
                sizes.push(if vertical {
                    vec2(BAR_LENGTH, BAR_THICKNESS)
                } else {
                    vec2(BAR_THICKNESS, BAR_LENGTH)
                });
            }
            StateKind::Simple => {
                let label = measure_block(
                    &display_label(state),
                    metrics.label_size,
                    metrics.line_height,
                    220.0,
                    measure,
                );
                let mut width = label.size.x;
                let mut height = label.size.y;
                if !state.description.is_empty() {
                    for line in &state.description {
                        width = width.max(measure(line, metrics.detail_size));
                    }
                    height +=
                        state.description.len() as f32 * metrics.detail_size * metrics.line_height
                            + 10.0;
                }
                sizes.push(vec2(
                    (width + metrics.node_padding_x * 2.0).max(metrics.node_min_width),
                    (height + metrics.node_padding_y * 1.6).max(metrics.node_min_height),
                ));
                blocks.push(Some(label));
            }
        }
    }

    let index_of = |id: &str| data.states.iter().position(|state| state.id == id);
    let mut edges = Vec::new();
    let mut edge_source = Vec::new();
    for (index, transition) in data.transitions.iter().enumerate() {
        if let (Some(from), Some(to)) = (index_of(&transition.from), index_of(&transition.to)) {
            edges.push((from, to));
            edge_source.push(index);
        }
    }

    let layout_pairs = edges.clone();
    let layout = layout_layered(&GraphInput {
        node_sizes: sizes.clone(),
        edges,
        direction: data.direction,
        rank_gap: metrics.rank_gap * 0.95,
        sibling_gap: metrics.sibling_gap,
        edge_lane: 18.0,
        node_group: Vec::new(),
        group_padding: 0.0,
    });

    let title_height = data
        .title
        .as_ref()
        .map(|_| metrics.title_size * metrics.line_height + metrics.margin * 0.6)
        .unwrap_or(0.0);
    let origin = vec2(metrics.margin, metrics.margin + title_height);
    scene.size = vec2(
        layout.size.x + metrics.margin * 2.0,
        layout.size.y + metrics.margin * 2.0 + title_height,
    );

    if let Some(title) = &data.title {
        push_label(
            &mut scene,
            title.clone(),
            vec2(metrics.margin, metrics.margin),
            label_style(
                metrics.title_size,
                theme.text,
                TextAlign::Left,
                TextBaseline::Top,
                LAYER_TITLE,
            ),
        );
    }

    let bounds_of = |index: usize| BoxBounds {
        position: layout.positions[index] + origin,
        size: sizes[index],
    };

    let offsets = lane_offsets(&layout_pairs, 20.0);
    for (path_index, waypoints) in layout.edge_waypoints.iter().enumerate() {
        let transition = &data.transitions[edge_source[path_index]];
        let (Some(from), Some(to)) = (index_of(&transition.from), index_of(&transition.to)) else {
            continue;
        };
        let points: Vec<Vec2> = waypoints.iter().map(|point| point + origin).collect();
        let width = width_for_style(transition.style, metrics.edge_width);
        let path = draw_layered_edge(
            &mut scene,
            &EdgeRoute {
                waypoints: &points,
                source: bounds_of(from),
                target: bounds_of(to),
                direction: data.direction,
                routing: crate::schema::EdgeRouting::Orthogonal,
                lane_offset: offsets[path_index],
            },
            &EdgeVisual {
                color: theme.edge,
                width,
                dash: dash_for_style(transition.style, width),
                head: ArrowHead::Arrow,
                tail: ArrowHead::None,
            },
            theme,
        );
        if let Some(label) = &transition.label {
            push_edge_label(
                &mut scene,
                label,
                &path,
                EdgeLabelStyle {
                    size: metrics.detail_size,
                    color: theme.text,
                    background: theme.edge_label_background,
                    lateral: offsets[path_index],
                },
                measure,
            );
        }
    }

    for (index, state) in data.states.iter().enumerate() {
        let bounds = bounds_of(index);
        let center = box_center(bounds);
        let colors = accent_colors(theme, state.accent);
        let neutral = matches!(state.accent, crate::schema::Accent::Neutral);
        match state.kind {
            StateKind::Start => push_circle(
                &mut scene,
                center,
                MARKER_RADIUS,
                paint_fill(theme.text, LAYER_NODE),
            ),
            StateKind::End => {
                push_circle(
                    &mut scene,
                    center,
                    MARKER_RADIUS,
                    paint_surface(theme.background, theme.text, 2.0, LAYER_NODE),
                );
                push_circle(
                    &mut scene,
                    center,
                    MARKER_RADIUS * 0.55,
                    paint_fill(theme.text, LAYER_NODE + 0.01),
                );
            }
            StateKind::Choice => {
                let half = bounds.size * 0.5;
                push_polygon(
                    &mut scene,
                    vec![
                        vec2(center.x, center.y - half.y),
                        vec2(center.x + half.x, center.y),
                        vec2(center.x, center.y + half.y),
                        vec2(center.x - half.x, center.y),
                    ],
                    paint_surface(
                        if neutral {
                            theme.surface_alt
                        } else {
                            colors.fill
                        },
                        if neutral { theme.border } else { colors.border },
                        metrics.border_width,
                        LAYER_NODE,
                    ),
                );
                if !state.label.is_empty() {
                    push_label(
                        &mut scene,
                        state.label.clone(),
                        vec2(bounds.position.x + bounds.size.x + 10.0, center.y),
                        label_style(
                            metrics.detail_size,
                            theme.text_muted,
                            TextAlign::Left,
                            TextBaseline::Middle,
                            LAYER_NODE_TEXT,
                        ),
                    );
                }
            }
            StateKind::Fork | StateKind::Join => {
                push_rect(
                    &mut scene,
                    bounds.position,
                    bounds.size,
                    2.0,
                    paint_fill(theme.text, LAYER_NODE),
                );
                if !state.label.is_empty() {
                    push_label(
                        &mut scene,
                        state.label.clone(),
                        vec2(center.x, bounds.position.y - 12.0),
                        label_style(
                            metrics.detail_size,
                            theme.text_muted,
                            TextAlign::Center,
                            TextBaseline::Bottom,
                            LAYER_NODE_TEXT,
                        ),
                    );
                }
            }
            StateKind::Simple => {
                push_rect(
                    &mut scene,
                    bounds.position,
                    bounds.size,
                    metrics.corner_radius * 1.4,
                    paint_surface(
                        if neutral { theme.surface } else { colors.fill },
                        if neutral { theme.border } else { colors.border },
                        metrics.border_width,
                        LAYER_NODE,
                    ),
                );
                let text_color = if neutral { theme.text } else { colors.text };
                let description_height = if state.description.is_empty() {
                    0.0
                } else {
                    state.description.len() as f32 * metrics.detail_size * metrics.line_height
                        + 10.0
                };
                if let Some(label) = blocks[index].as_ref() {
                    let step = metrics.label_size * metrics.line_height;
                    let top = center.y - (label.size.y + description_height) * 0.5;
                    for (line_index, line) in label.lines.iter().enumerate() {
                        push_label(
                            &mut scene,
                            line.clone(),
                            vec2(center.x, top + line_index as f32 * step + step * 0.5),
                            label_style(
                                metrics.label_size,
                                text_color,
                                TextAlign::Center,
                                TextBaseline::Middle,
                                LAYER_NODE_TEXT,
                            ),
                        );
                    }
                    if !state.description.is_empty() {
                        let divider_y = top + label.size.y + 5.0;
                        push_stroke(
                            &mut scene,
                            vec![
                                vec2(bounds.position.x, divider_y),
                                vec2(bounds.position.x + bounds.size.x, divider_y),
                            ],
                            1.2,
                            if neutral { theme.border } else { colors.border },
                            None,
                            LAYER_NODE + 0.02,
                        );
                        let detail_step = metrics.detail_size * metrics.line_height;
                        for (line_index, line) in state.description.iter().enumerate() {
                            push_label(
                                &mut scene,
                                line.clone(),
                                vec2(
                                    bounds.position.x + metrics.node_padding_x,
                                    divider_y
                                        + 5.0
                                        + line_index as f32 * detail_step
                                        + detail_step * 0.5,
                                ),
                                label_style(
                                    metrics.detail_size,
                                    theme.text_muted,
                                    TextAlign::Left,
                                    TextBaseline::Middle,
                                    LAYER_NODE_TEXT,
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    scene
}

fn display_label(state: &State) -> String {
    if state.label.is_empty() {
        state.id.clone()
    } else {
        state.label.clone()
    }
}
