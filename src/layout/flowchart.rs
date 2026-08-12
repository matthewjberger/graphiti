use crate::layout::edges::{EdgeRoute, EdgeVisual, draw_layered_edge, lane_offsets};
use crate::layout::graph::{GraphInput, layout_layered};
use crate::layout::node_shape::{push_node_shape, shape_size};
use crate::layout::route::{BoxBounds, EdgeLabelStyle, box_center, push_edge_label};
use crate::layout::text::{Measure, TextBlock, measure_block};
use crate::scene::{
    LAYER_GROUP, LAYER_GROUP_LABEL, LAYER_NODE, LAYER_NODE_TEXT, LAYER_TITLE, Scene, TextAlign,
    TextBaseline, dash_for_style, label_style, paint_surface, push_label, push_rect,
    width_for_style,
};
use crate::schema::flowchart::Flowchart;
use crate::theme::{Theme, accent_colors};
use nalgebra_glm::{Vec2, vec2};

const MAX_LABEL_WIDTH: f32 = 230.0;

pub fn generate(data: &Flowchart, theme: &Theme, measure: Measure) -> Scene {
    let metrics = theme.metrics;
    let mut scene = Scene {
        background: Some(theme.background),
        ..Scene::default()
    };

    let mut label_blocks: Vec<TextBlock> = Vec::with_capacity(data.nodes.len());
    let mut detail_blocks: Vec<Option<TextBlock>> = Vec::with_capacity(data.nodes.len());
    let mut sizes = Vec::with_capacity(data.nodes.len());
    for node in &data.nodes {
        let label = measure_block(
            &node.label,
            metrics.label_size,
            metrics.line_height,
            MAX_LABEL_WIDTH,
            measure,
        );
        let detail = node.detail.as_ref().map(|text| {
            measure_block(
                text,
                metrics.detail_size,
                metrics.line_height,
                MAX_LABEL_WIDTH,
                measure,
            )
        });
        let mut content = label.size;
        if let Some(detail) = &detail {
            content.x = content.x.max(detail.size.x);
            content.y += detail.size.y + 4.0;
        }
        sizes.push(shape_size(node.shape, content, &metrics));
        label_blocks.push(label);
        detail_blocks.push(detail);
    }

    let index_of = |id: &str| data.nodes.iter().position(|node| node.id == id);
    let mut edges = Vec::with_capacity(data.edges.len());
    let mut edge_source = Vec::with_capacity(data.edges.len());
    for (edge_index, edge) in data.edges.iter().enumerate() {
        if let (Some(from), Some(to)) = (index_of(&edge.from), index_of(&edge.to)) {
            edges.push((from, to));
            edge_source.push(edge_index);
        }
    }

    let mut node_group = vec![None; data.nodes.len()];
    for (group_index, group) in data.groups.iter().enumerate() {
        for member in &group.nodes {
            if let Some(node) = index_of(member) {
                node_group[node] = Some(group_index);
            }
        }
    }

    let sibling_gap = if data.groups.is_empty() {
        metrics.sibling_gap
    } else {
        metrics.sibling_gap + metrics.group_padding
    };
    let layout_pairs = edges.clone();
    let layout = layout_layered(&GraphInput {
        node_sizes: sizes.clone(),
        edges,
        direction: data.direction,
        rank_gap: metrics.rank_gap,
        sibling_gap,
        edge_lane: 18.0,
        node_group,
        group_padding: metrics.group_padding * 0.6 + 14.0,
    });

    let title_block = data.title.as_ref().map(|title| {
        measure_block(
            title,
            metrics.title_size,
            metrics.line_height,
            layout.size.x.max(320.0),
            measure,
        )
    });
    let title_height = title_block
        .as_ref()
        .map(|block| block.size.y + metrics.margin * 0.6)
        .unwrap_or(0.0);
    let group_inset = if data.groups.is_empty() {
        0.0
    } else {
        metrics.group_padding
    };
    let origin = vec2(
        metrics.margin + group_inset,
        metrics.margin + title_height + group_inset,
    );
    scene.size = vec2(
        layout.size.x + (metrics.margin + group_inset) * 2.0,
        layout.size.y + (metrics.margin + group_inset) * 2.0 + title_height,
    );

    if let Some(block) = &title_block {
        for (line_index, line) in block.lines.iter().enumerate() {
            push_label(
                &mut scene,
                line.clone(),
                vec2(
                    metrics.margin,
                    metrics.margin + line_index as f32 * metrics.title_size * metrics.line_height,
                ),
                label_style(
                    metrics.title_size,
                    theme.text,
                    TextAlign::Left,
                    TextBaseline::Top,
                    LAYER_TITLE,
                ),
            );
        }
    }

    let bounds_of = |node: usize| BoxBounds {
        position: layout.positions[node] + origin,
        size: sizes[node],
    };

    for (group_index, group) in data.groups.iter().enumerate() {
        let members: Vec<usize> = group
            .nodes
            .iter()
            .filter_map(|member| index_of(member))
            .collect();
        if members.is_empty() {
            continue;
        }
        let mut minimum = vec2(f32::MAX, f32::MAX);
        let mut maximum = vec2(f32::MIN, f32::MIN);
        for &member in &members {
            let bounds = bounds_of(member);
            minimum.x = minimum.x.min(bounds.position.x);
            minimum.y = minimum.y.min(bounds.position.y);
            maximum.x = maximum.x.max(bounds.position.x + bounds.size.x);
            maximum.y = maximum.y.max(bounds.position.y + bounds.size.y);
        }
        let label_space = if group.label.is_empty() { 0.0 } else { 26.0 };
        let padding = metrics.group_padding * 0.6;
        let position = vec2(minimum.x - padding, minimum.y - padding - label_space);
        let size = vec2(
            maximum.x - minimum.x + padding * 2.0,
            maximum.y - minimum.y + padding * 2.0 + label_space,
        );
        let colors = accent_colors(theme, group.accent);
        let border = if matches!(group.accent, crate::schema::Accent::Neutral) {
            theme.group_border
        } else {
            colors.border
        };
        push_rect(
            &mut scene,
            position,
            size,
            metrics.corner_radius * 1.5,
            paint_surface(
                theme.group_fill,
                border,
                metrics.border_width,
                LAYER_GROUP + group_index as f32 * 0.001,
            ),
        );
        if !group.label.is_empty() {
            push_label(
                &mut scene,
                group.label.clone(),
                vec2(position.x + 14.0, position.y + label_space * 0.5 + 4.0),
                label_style(
                    metrics.detail_size + 1.0,
                    theme.text_muted,
                    TextAlign::Left,
                    TextBaseline::Middle,
                    LAYER_GROUP_LABEL,
                ),
            );
        }
    }

    let offsets = lane_offsets(&layout_pairs, 20.0);
    for (path_index, waypoints) in layout.edge_waypoints.iter().enumerate() {
        let edge = &data.edges[edge_source[path_index]];
        let (Some(from), Some(to)) = (index_of(&edge.from), index_of(&edge.to)) else {
            continue;
        };
        let points: Vec<Vec2> = waypoints.iter().map(|point| point + origin).collect();
        let colors = accent_colors(theme, edge.accent);
        let color = if matches!(edge.accent, crate::schema::Accent::Neutral) {
            theme.edge
        } else {
            colors.strong
        };
        let width = width_for_style(edge.style, metrics.edge_width);
        let path = draw_layered_edge(
            &mut scene,
            &EdgeRoute {
                waypoints: &points,
                source: bounds_of(from),
                target: bounds_of(to),
                direction: data.direction,
                routing: data.routing,
                lane_offset: offsets[path_index],
            },
            &EdgeVisual {
                color,
                width,
                dash: dash_for_style(edge.style, width),
                head: edge.head,
                tail: edge.tail,
            },
            theme,
        );
        if let Some(label) = &edge.label {
            push_edge_label(
                &mut scene,
                label,
                &path,
                EdgeLabelStyle {
                    size: metrics.detail_size,
                    color: theme.text_muted,
                    background: theme.edge_label_background,
                    lateral: offsets[path_index],
                },
                measure,
            );
        }
    }

    for (node_index, node) in data.nodes.iter().enumerate() {
        let bounds = bounds_of(node_index);
        let colors = accent_colors(theme, node.accent);
        let neutral = matches!(node.accent, crate::schema::Accent::Neutral);
        let fill = if neutral { theme.surface } else { colors.fill };
        let border = if neutral { theme.border } else { colors.border };
        push_node_shape(
            &mut scene,
            node.shape,
            bounds.position,
            bounds.size,
            paint_surface(fill, border, metrics.border_width, LAYER_NODE),
            metrics.corner_radius,
        );

        let text_color = if neutral { theme.text } else { colors.text };
        let label = &label_blocks[node_index];
        let detail = &detail_blocks[node_index];
        let detail_height = detail
            .as_ref()
            .map(|block| block.size.y + 4.0)
            .unwrap_or(0.0);
        let center = box_center(bounds);
        let block_top = center.y - (label.size.y + detail_height) * 0.5;
        let step = metrics.label_size * metrics.line_height;
        for (line_index, line) in label.lines.iter().enumerate() {
            push_label(
                &mut scene,
                line.clone(),
                vec2(center.x, block_top + line_index as f32 * step + step * 0.5),
                label_style(
                    metrics.label_size,
                    text_color,
                    TextAlign::Center,
                    TextBaseline::Middle,
                    LAYER_NODE_TEXT,
                ),
            );
        }
        if let Some(detail) = detail {
            let detail_step = metrics.detail_size * metrics.line_height;
            let detail_top = block_top + label.size.y + 4.0;
            for (line_index, line) in detail.lines.iter().enumerate() {
                push_label(
                    &mut scene,
                    line.clone(),
                    vec2(
                        center.x,
                        detail_top + line_index as f32 * detail_step + detail_step * 0.5,
                    ),
                    label_style(
                        metrics.detail_size,
                        theme.text_muted,
                        TextAlign::Center,
                        TextBaseline::Middle,
                        LAYER_NODE_TEXT,
                    ),
                );
            }
        }
    }

    scene
}
