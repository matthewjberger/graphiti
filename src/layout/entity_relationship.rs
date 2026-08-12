use crate::layout::compartment::{
    Compartment, Content, Row, Style, draw_compartments, measure_compartments,
};
use crate::layout::edges::{EdgeRoute, EdgeVisual, draw_layered_edge, lane_offsets};
use crate::layout::graph::{GraphInput, layout_layered};
use crate::layout::route::{BoxBounds, EdgeLabelStyle, push_edge_label};
use crate::layout::text::Measure;
use crate::scene::{Dash, LAYER_TITLE, Scene, TextAlign, TextBaseline, label_style, push_label};
use crate::schema::ArrowHead;
use crate::schema::entity_relationship::{
    Attribute, Cardinality, Entity, EntityRelationship, KeyKind,
};
use crate::theme::{Theme, accent_colors};
use nalgebra_glm::vec2;

pub fn generate(data: &EntityRelationship, theme: &Theme, measure: Measure) -> Scene {
    let metrics = theme.metrics;
    let mut scene = Scene {
        background: Some(theme.background),
        ..Scene::default()
    };

    let mut contents = Vec::with_capacity(data.entities.len());
    let mut layouts = Vec::with_capacity(data.entities.len());
    let mut sizes = Vec::with_capacity(data.entities.len());
    for entity in &data.entities {
        let content = Content {
            header: vec![display_name(entity)],
            header_size: metrics.label_size + 1.0,
            subtitle: None,
            subtitle_size: metrics.detail_size,
            row_size: metrics.member_size,
            compartments: vec![Compartment {
                rows: entity.attributes.iter().map(attribute_row).collect(),
            }],
        };
        let layout = measure_compartments(&content, theme, measure);
        sizes.push(layout.size);
        layouts.push(layout);
        contents.push(content);
    }

    let index_of = |id: &str| {
        data.entities
            .iter()
            .position(|entity| entity.id == id || entity.name == id)
    };
    let mut edges = Vec::new();
    let mut edge_source = Vec::new();
    for (index, relationship) in data.relationships.iter().enumerate() {
        if let (Some(from), Some(to)) = (index_of(&relationship.from), index_of(&relationship.to)) {
            edges.push((from, to));
            edge_source.push(index);
        }
    }

    let layout_pairs = edges.clone();
    let layout = layout_layered(&GraphInput {
        node_sizes: sizes.clone(),
        edges,
        direction: data.direction,
        rank_gap: metrics.rank_gap * 1.3,
        sibling_gap: metrics.sibling_gap * 1.2,
        edge_lane: 34.0,
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
        let relationship = &data.relationships[edge_source[path_index]];
        let (Some(from), Some(to)) = (index_of(&relationship.from), index_of(&relationship.to))
        else {
            continue;
        };
        let points: Vec<_> = waypoints.iter().map(|point| point + origin).collect();
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
                width: metrics.edge_width,
                dash: if relationship.identifying {
                    None
                } else {
                    Some(Dash { on: 8.0, off: 6.0 })
                },
                head: crows_foot(relationship.to_cardinality),
                tail: crows_foot(relationship.from_cardinality),
            },
            theme,
        );
        if let Some(label) = &relationship.label {
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

    for (index, entity) in data.entities.iter().enumerate() {
        let colors = accent_colors(theme, entity.accent);
        let neutral = matches!(entity.accent, crate::schema::Accent::Neutral);
        draw_compartments(
            &mut scene,
            &layouts[index],
            layout.positions[index] + origin,
            &contents[index],
            &Style {
                fill: theme.surface,
                header_fill: if neutral {
                    theme.surface_alt
                } else {
                    colors.fill
                },
                border: if neutral { theme.border } else { colors.border },
                text: theme.text,
                muted: theme.text_muted,
                accent: if neutral {
                    theme.info.strong
                } else {
                    colors.strong
                },
            },
            theme,
        );
    }

    scene
}

fn display_name(entity: &Entity) -> String {
    if entity.name.is_empty() {
        entity.id.clone()
    } else {
        entity.name.clone()
    }
}

fn attribute_row(attribute: &Attribute) -> Row {
    let mut text = attribute.name.clone();
    if let Some(type_name) = &attribute.type_name {
        text = format!("{text}  {type_name}");
    }
    let badge = match attribute.key {
        KeyKind::None => attribute.comment.clone(),
        KeyKind::Primary => Some("PK".to_string()),
        KeyKind::Foreign => Some("FK".to_string()),
        KeyKind::Unique => Some("UK".to_string()),
    };
    Row {
        text,
        badge,
        muted: matches!(attribute.key, KeyKind::None),
    }
}

fn crows_foot(cardinality: Cardinality) -> ArrowHead {
    match cardinality {
        Cardinality::ExactlyOne => ArrowHead::CrowsFootOne,
        Cardinality::ZeroOrOne => ArrowHead::CrowsFootZeroOrOne,
        Cardinality::ZeroOrMany => ArrowHead::CrowsFootZeroOrMany,
        Cardinality::OneOrMany => ArrowHead::CrowsFootOneOrMany,
    }
}
