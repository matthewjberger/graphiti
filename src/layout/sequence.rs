use crate::layout::route::{ArrowStyle, push_arrow};
use crate::layout::text::{Measure, measure_block};
use crate::scene::{
    Dash, LAYER_EDGE, LAYER_EDGE_LABEL, LAYER_GROUP, LAYER_GROUP_LABEL, LAYER_NODE,
    LAYER_NODE_TEXT, LAYER_TITLE, Paint, Scene, TextAlign, TextBaseline, label_style,
    paint_outline, paint_surface, push_label, push_polygon, push_rect, push_stroke,
};
use crate::schema::ArrowHead;
use crate::schema::sequence::{
    Fragment, FragmentKind, Message, MessageKind, Note, NotePlacement, Participant,
    ParticipantKind, Sequence, SequenceStep,
};
use crate::theme::{Theme, accent_colors};
use nalgebra_glm::{Vec2, vec2};

const LANE_GAP: f32 = 56.0;
const MESSAGE_GAP: f32 = 48.0;
const SELF_MESSAGE_HEIGHT: f32 = 42.0;
const FRAGMENT_HEADER: f32 = 26.0;
const FRAGMENT_PADDING: f32 = 18.0;
const ACTIVATION_WIDTH: f32 = 12.0;

struct Activation {
    lane: usize,
    start: f32,
    end: f32,
    depth: usize,
}

struct Context<'a> {
    theme: &'a Theme,
    positions: Vec<f32>,
    widths: Vec<f32>,
    heights: Vec<f32>,
    activations: Vec<Activation>,
    open: Vec<Vec<f32>>,
}

pub fn generate(data: &Sequence, theme: &Theme, measure: Measure) -> Scene {
    let metrics = theme.metrics;
    let mut scene = Scene {
        background: Some(theme.background),
        ..Scene::default()
    };

    if data.participants.is_empty() {
        scene.size = vec2(320.0, 160.0);
        return scene;
    }

    let mut widths = Vec::with_capacity(data.participants.len());
    let mut heights = Vec::with_capacity(data.participants.len());
    for participant in &data.participants {
        let block = measure_block(
            &display_name(participant),
            metrics.label_size,
            metrics.line_height,
            200.0,
            measure,
        );
        widths.push((block.size.x + metrics.node_padding_x * 2.0).max(120.0));
        heights.push((block.size.y + metrics.node_padding_y * 1.6).max(46.0));
    }

    let mut gaps = vec![LANE_GAP; data.participants.len().saturating_sub(1)];
    collect_gaps(&data.steps, data, metrics.detail_size, measure, &mut gaps);

    let title_height = data
        .title
        .as_ref()
        .map(|_| metrics.title_size * metrics.line_height + metrics.margin * 0.6)
        .unwrap_or(0.0);
    let header_height = heights.iter().copied().fold(0.0f32, f32::max);
    let left = metrics.margin + widths[0] * 0.5;

    let mut positions = Vec::with_capacity(data.participants.len());
    let mut cursor = left;
    for index in 0..data.participants.len() {
        if index > 0 {
            cursor += widths[index - 1] * 0.5 + widths[index] * 0.5 + gaps[index - 1];
        }
        positions.push(cursor);
    }

    let top = metrics.margin + title_height;
    let mut context = Context {
        theme,
        positions,
        widths,
        heights,
        activations: Vec::new(),
        open: vec![Vec::new(); data.participants.len()],
    };

    let lifeline_top = top + header_height;
    let mut cursor_y = lifeline_top + MESSAGE_GAP * 0.6;
    emit_steps(
        &data.steps,
        data,
        &mut scene,
        &mut context,
        &mut cursor_y,
        0,
        measure,
    );
    let lifeline_bottom = cursor_y + MESSAGE_GAP * 0.5;

    for lane in 0..data.participants.len() {
        let x = context.positions[lane];
        push_stroke(
            &mut scene,
            vec![vec2(x, lifeline_top), vec2(x, lifeline_bottom)],
            1.4,
            theme.border,
            Some(Dash { on: 7.0, off: 6.0 }),
            LAYER_GROUP_LABEL,
        );
    }

    for lane in 0..data.participants.len() {
        let pending: Vec<f32> = context.open[lane].drain(..).collect();
        for start in pending {
            context.activations.push(Activation {
                lane,
                start,
                end: lifeline_bottom,
                depth: 0,
            });
        }
    }
    let activations = std::mem::take(&mut context.activations);
    for activation in activations {
        let x = context.positions[activation.lane];
        let offset = activation.depth as f32 * ACTIVATION_WIDTH * 0.6;
        push_rect(
            &mut scene,
            vec2(x - ACTIVATION_WIDTH * 0.5 + offset, activation.start),
            vec2(
                ACTIVATION_WIDTH,
                (activation.end - activation.start).max(8.0),
            ),
            2.0,
            paint_surface(theme.surface_alt, theme.border, 1.2, LAYER_NODE - 0.5),
        );
    }

    for (lane, participant) in data.participants.iter().enumerate() {
        let colors = accent_colors(theme, participant.accent);
        let neutral = matches!(participant.accent, crate::schema::Accent::Neutral);
        let width = context.widths[lane];
        let height = context.heights[lane];
        let position = vec2(
            context.positions[lane] - width * 0.5,
            top + header_height - height,
        );
        let paint = paint_surface(
            if neutral { theme.surface } else { colors.fill },
            if neutral { theme.border } else { colors.border },
            metrics.border_width,
            LAYER_NODE,
        );
        draw_participant(
            &mut scene,
            participant,
            position,
            vec2(width, height),
            paint,
            metrics.corner_radius,
        );
        push_label(
            &mut scene,
            display_name(participant),
            vec2(context.positions[lane], position.y + height * 0.5),
            label_style(
                metrics.label_size,
                if neutral { theme.text } else { colors.text },
                TextAlign::Center,
                TextBaseline::Middle,
                LAYER_NODE_TEXT,
            ),
        );
    }

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

    let last = data.participants.len() - 1;
    scene.size = vec2(
        context.positions[last] + context.widths[last] * 0.5 + metrics.margin,
        lifeline_bottom + metrics.margin,
    );
    scene
}

fn display_name(participant: &Participant) -> String {
    if participant.label.is_empty() {
        participant.id.clone()
    } else {
        participant.label.clone()
    }
}

fn draw_participant(
    scene: &mut Scene,
    participant: &Participant,
    position: Vec2,
    size: Vec2,
    paint: Paint,
    corner_radius: f32,
) {
    let border = paint.stroke.unwrap_or_default();
    match participant.kind {
        ParticipantKind::Participant => {
            push_rect(scene, position, size, corner_radius, paint);
        }
        ParticipantKind::Actor => {
            push_rect(scene, position, size, size.y * 0.5, paint);
        }
        ParticipantKind::Boundary => {
            push_rect(scene, position, size, 2.0, paint);
            push_stroke(
                scene,
                vec![
                    vec2(position.x, position.y + size.y * 0.5),
                    vec2(position.x - 10.0, position.y + size.y * 0.5),
                ],
                paint.stroke_width,
                border,
                None,
                LAYER_NODE,
            );
        }
        ParticipantKind::Database => {
            let lip = 7.0;
            push_rect(scene, position, size, lip, paint);
            push_stroke(
                scene,
                vec![
                    vec2(position.x + 6.0, position.y + lip * 1.6),
                    vec2(position.x + size.x - 6.0, position.y + lip * 1.6),
                ],
                paint.stroke_width,
                border,
                None,
                LAYER_NODE + 0.01,
            );
        }
    }
}

fn collect_gaps(
    steps: &[SequenceStep],
    data: &Sequence,
    font_size: f32,
    measure: Measure,
    gaps: &mut Vec<f32>,
) {
    for step in steps {
        match step {
            SequenceStep::Message(message) => {
                let from = lane_of(data, &message.from);
                let to = lane_of(data, &message.to);
                if let (Some(from), Some(to)) = (from, to)
                    && from != to
                {
                    let width = measure(&message.label, font_size) + 34.0;
                    let low = from.min(to);
                    let high = from.max(to);
                    let per_gap = width / (high - low) as f32;
                    for gap in gaps.iter_mut().take(high).skip(low) {
                        if *gap < per_gap {
                            *gap = per_gap;
                        }
                    }
                }
            }
            SequenceStep::Fragment(fragment) => {
                collect_gaps(&fragment.steps, data, font_size, measure, gaps);
                for branch in &fragment.branches {
                    collect_gaps(&branch.steps, data, font_size, measure, gaps);
                }
            }
            _ => {}
        }
    }
}

fn lane_of(data: &Sequence, id: &str) -> Option<usize> {
    data.participants
        .iter()
        .position(|participant| participant.id == id)
}

fn emit_steps(
    steps: &[SequenceStep],
    data: &Sequence,
    scene: &mut Scene,
    context: &mut Context,
    cursor: &mut f32,
    depth: usize,
    measure: Measure,
) {
    for step in steps {
        match step {
            SequenceStep::Message(message) => {
                emit_message(message, data, scene, context, cursor, measure)
            }
            SequenceStep::Note(note) => emit_note(note, data, scene, context, cursor, measure),
            SequenceStep::Divider(divider) => {
                emit_divider(&divider.label, scene, context, cursor, measure)
            }
            SequenceStep::Fragment(fragment) => {
                emit_fragment(fragment, data, scene, context, cursor, depth, measure)
            }
        }
    }
}

fn emit_divider(
    label: &str,
    scene: &mut Scene,
    context: &mut Context,
    cursor: &mut f32,
    measure: Measure,
) {
    let metrics = context.theme.metrics;
    *cursor += MESSAGE_GAP * 0.4;
    let left = context.positions[0] - context.widths[0] * 0.5;
    let last = context.positions.len() - 1;
    let right = context.positions[last] + context.widths[last] * 0.5;
    push_stroke(
        scene,
        vec![vec2(left, *cursor), vec2(right, *cursor)],
        1.4,
        context.theme.border,
        Some(Dash { on: 9.0, off: 7.0 }),
        LAYER_EDGE,
    );
    if !label.is_empty() {
        let width = measure(label, metrics.detail_size) + 20.0;
        let center = (left + right) * 0.5;
        push_rect(
            scene,
            vec2(center - width * 0.5, *cursor - metrics.detail_size),
            vec2(width, metrics.detail_size * 2.0),
            4.0,
            paint_surface(
                context.theme.background,
                context.theme.border,
                1.2,
                LAYER_EDGE_LABEL - 0.5,
            ),
        );
        push_label(
            scene,
            label.to_string(),
            vec2(center, *cursor),
            label_style(
                metrics.detail_size,
                context.theme.text_muted,
                TextAlign::Center,
                TextBaseline::Middle,
                LAYER_EDGE_LABEL,
            ),
        );
    }
    *cursor += MESSAGE_GAP * 0.6;
}

fn emit_message(
    message: &Message,
    data: &Sequence,
    scene: &mut Scene,
    context: &mut Context,
    cursor: &mut f32,
    measure: Measure,
) {
    let metrics = context.theme.metrics;
    let (Some(from), Some(to)) = (lane_of(data, &message.from), lane_of(data, &message.to)) else {
        return;
    };
    let colors = accent_colors(context.theme, message.accent);
    let neutral = matches!(message.accent, crate::schema::Accent::Neutral);
    let color = if neutral {
        context.theme.edge
    } else {
        colors.strong
    };
    let dash = match message.kind {
        MessageKind::Reply => Some(Dash { on: 7.0, off: 5.0 }),
        _ => None,
    };
    let head = match message.kind {
        MessageKind::Async | MessageKind::Reply => ArrowHead::Open,
        MessageKind::Destroy => ArrowHead::Bar,
        _ => ArrowHead::Arrow,
    };
    let arrow = ArrowStyle {
        size: metrics.arrow_size,
        color,
        background: context.theme.background,
        width: metrics.edge_width,
        depth: LAYER_EDGE,
    };

    if message.activate {
        context.open[to].push(*cursor + 2.0);
    }

    if from == to {
        let x = context.positions[from];
        let out = x + 62.0;
        let top = *cursor;
        let bottom = *cursor + SELF_MESSAGE_HEIGHT;
        push_stroke(
            scene,
            vec![
                vec2(x + ACTIVATION_WIDTH * 0.5, top),
                vec2(out, top),
                vec2(out, bottom),
                vec2(
                    x + ACTIVATION_WIDTH * 0.5 + metrics.arrow_size * 0.8,
                    bottom,
                ),
            ],
            metrics.edge_width,
            color,
            dash,
            LAYER_EDGE,
        );
        push_arrow(
            scene,
            head,
            vec2(x + ACTIVATION_WIDTH * 0.5, bottom),
            vec2(-1.0, 0.0),
            arrow,
        );
        if !message.label.is_empty() {
            push_label(
                scene,
                message.label.clone(),
                vec2(out + 12.0, (top + bottom) * 0.5),
                label_style(
                    metrics.detail_size,
                    context.theme.text,
                    TextAlign::Left,
                    TextBaseline::Middle,
                    LAYER_EDGE_LABEL,
                ),
            );
        }
        *cursor = bottom + MESSAGE_GAP * 0.6;
    } else {
        let forward = context.positions[to] > context.positions[from];
        let sign = if forward { 1.0 } else { -1.0 };
        let start = vec2(
            context.positions[from] + sign * ACTIVATION_WIDTH * 0.5,
            *cursor,
        );
        let end = vec2(
            context.positions[to] - sign * ACTIVATION_WIDTH * 0.5,
            *cursor,
        );
        let trim = crate::layout::route::arrow_trim(head, metrics.arrow_size);
        push_stroke(
            scene,
            vec![start, vec2(end.x - sign * trim, end.y)],
            metrics.edge_width,
            color,
            dash,
            LAYER_EDGE,
        );
        push_arrow(scene, head, end, vec2(sign, 0.0), arrow);
        if !message.label.is_empty() {
            let block = measure_block(
                &message.label,
                metrics.detail_size,
                metrics.line_height,
                (end.x - start.x).abs().max(90.0),
                measure,
            );
            let center = (start.x + end.x) * 0.5;
            let step = metrics.detail_size * metrics.line_height;
            for (index, line) in block.lines.iter().enumerate() {
                let rows_below = (block.lines.len() - index - 1) as f32;
                push_label(
                    scene,
                    line.clone(),
                    vec2(center, *cursor - 10.0 - rows_below * step),
                    label_style(
                        metrics.detail_size,
                        context.theme.text,
                        TextAlign::Center,
                        TextBaseline::Bottom,
                        LAYER_EDGE_LABEL,
                    ),
                );
            }
            *cursor += (block.lines.len().saturating_sub(1)) as f32 * step;
        }
        *cursor += MESSAGE_GAP;
    }

    if message.deactivate {
        for lane in [from, to] {
            if let Some(start) = context.open[lane].pop() {
                let depth = context.open[lane].len();
                context.activations.push(Activation {
                    lane,
                    start,
                    end: *cursor - MESSAGE_GAP * 0.5,
                    depth,
                });
            }
        }
    }
}

fn emit_note(
    note: &Note,
    data: &Sequence,
    scene: &mut Scene,
    context: &mut Context,
    cursor: &mut f32,
    measure: Measure,
) {
    let metrics = context.theme.metrics;
    let lanes: Vec<usize> = note
        .over
        .iter()
        .filter_map(|id| lane_of(data, id))
        .collect();
    let block = measure_block(
        &note.text,
        metrics.detail_size,
        metrics.line_height,
        250.0,
        measure,
    );
    let width = block.size.x + 26.0;
    let height = block.size.y + 20.0;
    let anchor = if lanes.is_empty() {
        context.positions[0]
    } else {
        let low = lanes.iter().copied().min().unwrap();
        let high = lanes.iter().copied().max().unwrap();
        (context.positions[low] + context.positions[high]) * 0.5
    };
    let position = match note.placement {
        NotePlacement::Over => vec2(anchor - width * 0.5, *cursor),
        NotePlacement::Left => vec2(anchor - width - 26.0, *cursor),
        NotePlacement::Right => vec2(anchor + 26.0, *cursor),
    };
    let fold = 14.0f32.min(width * 0.3);
    push_polygon(
        scene,
        vec![
            position,
            vec2(position.x + width - fold, position.y),
            vec2(position.x + width, position.y + fold),
            vec2(position.x + width, position.y + height),
            vec2(position.x, position.y + height),
        ],
        paint_surface(
            context.theme.warning.fill,
            context.theme.warning.border,
            metrics.border_width,
            LAYER_NODE,
        ),
    );
    let step = metrics.detail_size * metrics.line_height;
    for (index, line) in block.lines.iter().enumerate() {
        push_label(
            scene,
            line.clone(),
            vec2(
                position.x + 13.0,
                position.y + 10.0 + index as f32 * step + step * 0.5,
            ),
            label_style(
                metrics.detail_size,
                context.theme.warning.text,
                TextAlign::Left,
                TextBaseline::Middle,
                LAYER_NODE_TEXT,
            ),
        );
    }
    *cursor += height + MESSAGE_GAP * 0.5;
}

fn emit_fragment(
    fragment: &Fragment,
    data: &Sequence,
    scene: &mut Scene,
    context: &mut Context,
    cursor: &mut f32,
    depth: usize,
    measure: Measure,
) {
    let metrics = context.theme.metrics;
    let start_y = *cursor;
    *cursor += FRAGMENT_HEADER + FRAGMENT_PADDING * 0.5;
    emit_steps(
        &fragment.steps,
        data,
        scene,
        context,
        cursor,
        depth + 1,
        measure,
    );
    let mut dividers = Vec::new();
    for branch in &fragment.branches {
        dividers.push((*cursor, branch.label.clone()));
        *cursor += FRAGMENT_PADDING;
        emit_steps(
            &branch.steps,
            data,
            scene,
            context,
            cursor,
            depth + 1,
            measure,
        );
    }
    let end_y = *cursor + FRAGMENT_PADDING * 0.25;

    let inset = depth as f32 * 8.0;
    let left = context.positions[0] - context.widths[0] * 0.5 - 14.0 + inset;
    let last = context.positions.len() - 1;
    let right = context.positions[last] + context.widths[last] * 0.5 + 14.0 - inset;

    push_rect(
        scene,
        vec2(left, start_y),
        vec2(right - left, end_y - start_y),
        4.0,
        paint_outline(
            context.theme.border,
            metrics.border_width,
            LAYER_GROUP + depth as f32 * 0.01,
        ),
    );

    let kind_label = fragment_label(fragment.kind);
    let kind_width = measure(kind_label, metrics.detail_size) + 20.0;
    push_polygon(
        scene,
        vec![
            vec2(left, start_y),
            vec2(left + kind_width, start_y),
            vec2(left + kind_width, start_y + FRAGMENT_HEADER - 6.0),
            vec2(left + kind_width - 10.0, start_y + FRAGMENT_HEADER),
            vec2(left, start_y + FRAGMENT_HEADER),
        ],
        paint_surface(
            context.theme.surface_alt,
            context.theme.border,
            metrics.border_width,
            LAYER_GROUP_LABEL + depth as f32 * 0.01,
        ),
    );
    push_label(
        scene,
        kind_label,
        vec2(left + 10.0, start_y + FRAGMENT_HEADER * 0.5),
        label_style(
            metrics.detail_size,
            context.theme.text_muted,
            TextAlign::Left,
            TextBaseline::Middle,
            LAYER_NODE_TEXT,
        ),
    );
    if !fragment.label.is_empty() {
        push_label(
            scene,
            format!("[{}]", fragment.label),
            vec2(left + kind_width + 12.0, start_y + FRAGMENT_HEADER * 0.5),
            label_style(
                metrics.detail_size,
                context.theme.text,
                TextAlign::Left,
                TextBaseline::Middle,
                LAYER_NODE_TEXT,
            ),
        );
    }

    for (y, label) in dividers {
        push_stroke(
            scene,
            vec![vec2(left, y), vec2(right, y)],
            1.3,
            context.theme.border,
            Some(Dash { on: 7.0, off: 6.0 }),
            LAYER_GROUP_LABEL,
        );
        if !label.is_empty() {
            push_label(
                scene,
                format!("[{label}]"),
                vec2(left + 12.0, y + 12.0),
                label_style(
                    metrics.detail_size,
                    context.theme.text,
                    TextAlign::Left,
                    TextBaseline::Middle,
                    LAYER_NODE_TEXT,
                ),
            );
        }
    }

    *cursor = end_y + MESSAGE_GAP * 0.4;
}

fn fragment_label(kind: FragmentKind) -> &'static str {
    match kind {
        FragmentKind::Loop => "loop",
        FragmentKind::Alt => "alt",
        FragmentKind::Opt => "opt",
        FragmentKind::Par => "par",
        FragmentKind::Critical => "critical",
        FragmentKind::Break => "break",
    }
}
