use crate::layout::compartment::{
    Compartment, Content, Row, Style, draw_compartments, measure_compartments,
};
use crate::layout::edges::{EdgeRoute, EdgeVisual, draw_layered_edge, lane_offsets};
use crate::layout::graph::{GraphInput, layout_layered};
use crate::layout::route::{BoxBounds, EdgeLabelStyle, push_edge_label, push_end_label};
use crate::layout::text::Measure;
use crate::scene::{Dash, LAYER_TITLE, Scene, TextAlign, TextBaseline, label_style, push_label};
use crate::schema::class_diagram::{Class, ClassDiagram, Member, RelationKind};
use crate::schema::{ArrowHead, Visibility};
use crate::theme::{Theme, accent_colors};
use nalgebra_glm::vec2;

pub fn generate(data: &ClassDiagram, theme: &Theme, measure: Measure) -> Scene {
    let metrics = theme.metrics;
    let mut scene = Scene {
        background: Some(theme.background),
        ..Scene::default()
    };

    let mut contents = Vec::with_capacity(data.classes.len());
    let mut layouts = Vec::with_capacity(data.classes.len());
    let mut sizes = Vec::with_capacity(data.classes.len());
    for class in &data.classes {
        let content = Content {
            header: vec![display_name(class)],
            header_size: metrics.label_size + 1.0,
            subtitle: class
                .stereotype
                .as_ref()
                .map(|stereotype| format!("«{stereotype}»")),
            subtitle_size: metrics.detail_size,
            row_size: metrics.member_size,
            compartments: vec![
                Compartment {
                    rows: class.fields.iter().map(member_row).collect(),
                },
                Compartment {
                    rows: class.methods.iter().map(method_row).collect(),
                },
            ],
        };
        let layout = measure_compartments(&content, theme, measure);
        sizes.push(layout.size);
        layouts.push(layout);
        contents.push(content);
    }

    let index_of = |id: &str| {
        data.classes
            .iter()
            .position(|class| class.id == id || class.name == id)
    };
    let mut edges = Vec::new();
    let mut edge_source = Vec::new();
    for (index, relation) in data.relations.iter().enumerate() {
        if let (Some(from), Some(to)) = (index_of(&relation.from), index_of(&relation.to)) {
            edges.push(layer_order(relation.kind, from, to));
            edge_source.push(index);
        }
    }

    let layout_pairs = edges.clone();
    let layout = layout_layered(&GraphInput {
        node_sizes: sizes.clone(),
        edges,
        direction: data.direction,
        rank_gap: metrics.rank_gap * 1.1,
        sibling_gap: metrics.sibling_gap * 1.1,
        edge_lane: 20.0,
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
        let relation = &data.relations[edge_source[path_index]];
        let (Some(from), Some(to)) = (index_of(&relation.from), index_of(&relation.to)) else {
            continue;
        };
        let (tail_index, head_index) = layer_order(relation.kind, from, to);
        let points: Vec<_> = waypoints.iter().map(|point| point + origin).collect();
        let path = draw_layered_edge(
            &mut scene,
            &EdgeRoute {
                waypoints: &points,
                source: bounds_of(tail_index),
                target: bounds_of(head_index),
                direction: data.direction,
                routing: crate::schema::EdgeRouting::Orthogonal,
                lane_offset: offsets[path_index],
            },
            &relation_visual(relation.kind, theme),
            theme,
        );
        if let Some(label) = &relation.label {
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
        let (start_cardinality, end_cardinality) = if tail_index == from {
            (
                relation.from_cardinality.as_deref(),
                relation.to_cardinality.as_deref(),
            )
        } else {
            (
                relation.to_cardinality.as_deref(),
                relation.from_cardinality.as_deref(),
            )
        };
        if let Some(text) = start_cardinality {
            push_end_label(
                &mut scene,
                text,
                &path,
                true,
                metrics.detail_size,
                theme.text_muted,
            );
        }
        if let Some(text) = end_cardinality {
            push_end_label(
                &mut scene,
                text,
                &path,
                false,
                metrics.detail_size,
                theme.text_muted,
            );
        }
    }

    for (index, class) in data.classes.iter().enumerate() {
        let colors = accent_colors(theme, class.accent);
        let neutral = matches!(class.accent, crate::schema::Accent::Neutral);
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
                accent: if neutral { theme.border } else { colors.strong },
            },
            theme,
        );
    }

    scene
}

fn display_name(class: &Class) -> String {
    if class.name.is_empty() {
        class.id.clone()
    } else {
        class.name.clone()
    }
}

fn visibility_symbol(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Public => "+",
        Visibility::Private => "-",
        Visibility::Protected => "#",
        Visibility::Package => "~",
    }
}

fn member_row(member: &Member) -> Row {
    let mut text = format!("{} {}", visibility_symbol(member.visibility), member.name);
    if let Some(type_name) = &member.type_name {
        text.push_str(&format!(": {type_name}"));
    }
    Row {
        text,
        badge: member.is_static.then(|| "static".to_string()),
        muted: false,
    }
}

fn method_row(member: &Member) -> Row {
    let mut text = format!("{} {}()", visibility_symbol(member.visibility), member.name);
    if let Some(type_name) = &member.type_name {
        text.push_str(&format!(": {type_name}"));
    }
    let badge = if member.is_abstract {
        Some("abstract".to_string())
    } else {
        member.is_static.then(|| "static".to_string())
    };
    Row {
        text,
        badge,
        muted: member.is_abstract,
    }
}

fn layer_order(kind: RelationKind, from: usize, to: usize) -> (usize, usize) {
    match kind {
        RelationKind::Inheritance | RelationKind::Realization => (to, from),
        _ => (from, to),
    }
}

fn relation_visual(kind: RelationKind, theme: &Theme) -> EdgeVisual {
    let width = theme.metrics.edge_width;
    let dashed = Some(Dash { on: 8.0, off: 6.0 });
    match kind {
        RelationKind::Inheritance => EdgeVisual {
            color: theme.edge,
            width,
            dash: None,
            head: ArrowHead::None,
            tail: ArrowHead::HollowTriangle,
        },
        RelationKind::Realization => EdgeVisual {
            color: theme.edge,
            width,
            dash: dashed,
            head: ArrowHead::None,
            tail: ArrowHead::HollowTriangle,
        },
        RelationKind::Composition => EdgeVisual {
            color: theme.edge,
            width,
            dash: None,
            head: ArrowHead::None,
            tail: ArrowHead::Diamond,
        },
        RelationKind::Aggregation => EdgeVisual {
            color: theme.edge,
            width,
            dash: None,
            head: ArrowHead::None,
            tail: ArrowHead::HollowDiamond,
        },
        RelationKind::Dependency => EdgeVisual {
            color: theme.edge,
            width,
            dash: dashed,
            head: ArrowHead::Open,
            tail: ArrowHead::None,
        },
        RelationKind::Association => EdgeVisual {
            color: theme.edge,
            width,
            dash: None,
            head: ArrowHead::Open,
            tail: ArrowHead::None,
        },
    }
}
