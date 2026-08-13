use graphiti::schema::common::{AccentOverride, Palette, Style};
use graphiti::schema::{self};
use leptos::prelude::*;

use crate::document::{Document, edit, read};
use crate::form::{Choice, NumberInput, TextInput, Toggle, optional};
use crate::options::{THEMES, theme_name};

#[derive(Clone, Copy, PartialEq)]
enum Metric {
    Zoom,
    LabelSize,
    TitleSize,
    DetailSize,
    NodePadding,
    NodeMinWidth,
    NodeMinHeight,
    RankGap,
    SiblingGap,
    CornerRadius,
    BorderWidth,
    EdgeWidth,
    ArrowSize,
    Margin,
    LineHeight,
}

const METRICS: &[(&str, Metric, &str)] = &[
    ("Zoom", Metric::Zoom, "1"),
    ("Label size", Metric::LabelSize, "17"),
    ("Title size", Metric::TitleSize, "30"),
    ("Detail size", Metric::DetailSize, "13"),
    ("Node padding", Metric::NodePadding, "22"),
    ("Node min width", Metric::NodeMinWidth, "104"),
    ("Node min height", Metric::NodeMinHeight, "52"),
    ("Rank gap", Metric::RankGap, "84"),
    ("Sibling gap", Metric::SiblingGap, "44"),
    ("Corner radius", Metric::CornerRadius, "8"),
    ("Border width", Metric::BorderWidth, "1.6"),
    ("Edge width", Metric::EdgeWidth, "1.8"),
    ("Arrow size", Metric::ArrowSize, "11"),
    ("Margin", Metric::Margin, "48"),
    ("Line height", Metric::LineHeight, "1.35"),
];

#[derive(Clone, Copy, PartialEq)]
enum Base {
    Background,
    Surface,
    SurfaceAlt,
    Border,
    Text,
    TextMuted,
    Edge,
    GroupFill,
    GroupBorder,
}

const BASES: &[(&str, Base, &str)] = &[
    ("Background", Base::Background, "#FCFCFD"),
    ("Surface", Base::Surface, "#FFFFFF"),
    ("Surface alt", Base::SurfaceAlt, "#F1F4F9"),
    ("Border", Base::Border, "#C9D2E0"),
    ("Text", Base::Text, "#1B2330"),
    ("Text muted", Base::TextMuted, "#64748B"),
    ("Edge", Base::Edge, "#8794A8"),
    ("Group fill", Base::GroupFill, "#F4F6FA"),
    ("Group border", Base::GroupBorder, "#D5DDE9"),
];

#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Primary,
    Success,
    Warning,
    Danger,
    Info,
    Muted,
}

const SLOTS: &[(&str, Slot)] = &[
    ("Primary", Slot::Primary),
    ("Success", Slot::Success),
    ("Warning", Slot::Warning),
    ("Danger", Slot::Danger),
    ("Info", Slot::Info),
    ("Muted", Slot::Muted),
];

#[derive(Clone, Copy, PartialEq)]
enum Part {
    Fill,
    Border,
    Strong,
    Text,
}

const PARTS: &[(&str, Part)] = &[
    ("Fill", Part::Fill),
    ("Border", Part::Border),
    ("Strong", Part::Strong),
    ("Text", Part::Text),
];

#[component]
pub fn StyleBlock(document: Document) -> impl IntoView {
    view! {
        <details class="section">
            <summary>"Style"</summary>
            <div class="panel-hint">
                "Everything here is optional. Empty means the base theme decides."
            </div>
            <div class="grid">
                <Choice
                    label="Theme"
                    options=THEMES
                    value=Signal::derive(move || {
                        with(document, |style| theme_name(style.theme.as_deref()))
                    })
                    change=Callback::new(move |chosen: Option<&'static str>| {
                        change(document, move |style| {
                            style.theme = chosen.map(str::to_string);
                        })
                    })
                />
                {METRICS.iter().map(|entry| metric_field(document, *entry)).collect_view()}
            </div>
            <div class="row">
                <Toggle
                    label="Compact"
                    value=Signal::derive(move || with(document, |style| style.compact))
                    change=Callback::new(move |value: bool| {
                        change(document, move |style| style.compact = value)
                    })
                />
                <Toggle
                    label="Monospace"
                    value=Signal::derive(move || with(document, |style| style.monospace))
                    change=Callback::new(move |value: bool| {
                        change(document, move |style| style.monospace = value)
                    })
                />
            </div>
            <details class="nested">
                <summary>"Palette"</summary>
                <div class="grid">
                    {BASES.iter().map(|entry| base_field(document, *entry)).collect_view()}
                </div>
                {SLOTS
                    .iter()
                    .map(|(name, slot)| {
                        let slot = *slot;
                        view! {
                            <details class="nested">
                                <summary>{*name}</summary>
                                <div class="grid">
                                    {PARTS
                                        .iter()
                                        .map(|part| accent_field(document, slot, *part))
                                        .collect_view()}
                                </div>
                            </details>
                        }
                    })
                    .collect_view()}
            </details>
        </details>
    }
}

fn metric_field(document: Document, entry: (&'static str, Metric, &'static str)) -> impl IntoView {
    let (label, metric, placeholder) = entry;
    view! {
        <NumberInput
            label=label
            placeholder=placeholder
            value=Signal::derive(move || with(document, |style| *metric_of(style, metric)))
            change=Callback::new(move |value: Option<f32>| {
                change(document, move |style| *metric_mut(style, metric) = value)
            })
        />
    }
}

fn base_field(document: Document, entry: (&'static str, Base, &'static str)) -> impl IntoView {
    let (label, base, placeholder) = entry;
    view! {
        <TextInput
            label=label
            placeholder=placeholder
            value=Signal::derive(move || {
                with(document, |style| base_of(&style.palette, base).clone().unwrap_or_default())
            })
            change=Callback::new(move |text: String| {
                change(document, move |style| {
                    *base_mut(&mut style.palette, base) = optional(text);
                })
            })
        />
    }
}

fn accent_field(document: Document, slot: Slot, part: (&'static str, Part)) -> impl IntoView {
    let (label, which) = part;
    view! {
        <TextInput
            label=label
            value=Signal::derive(move || {
                with(document, |style| {
                    part_of(accent_of(&style.palette, slot), which).clone().unwrap_or_default()
                })
            })
            change=Callback::new(move |text: String| {
                change(document, move |style| {
                    *part_mut(accent_mut(&mut style.palette, slot), which) = optional(text);
                })
            })
        />
    }
}

fn with<T: Default>(document: Document, take: impl FnOnce(&Style) -> T) -> T {
    read(document, |kind| take(schema::style(kind)))
}

fn change(document: Document, mutate: impl FnOnce(&mut Style)) {
    edit(document, |diagram| {
        mutate(schema::style_mut(&mut diagram.kind))
    });
}

fn metric_of(style: &Style, metric: Metric) -> &Option<f32> {
    match metric {
        Metric::Zoom => &style.zoom,
        Metric::LabelSize => &style.label_size,
        Metric::TitleSize => &style.title_size,
        Metric::DetailSize => &style.detail_size,
        Metric::NodePadding => &style.node_padding,
        Metric::NodeMinWidth => &style.node_min_width,
        Metric::NodeMinHeight => &style.node_min_height,
        Metric::RankGap => &style.rank_gap,
        Metric::SiblingGap => &style.sibling_gap,
        Metric::CornerRadius => &style.corner_radius,
        Metric::BorderWidth => &style.border_width,
        Metric::EdgeWidth => &style.edge_width,
        Metric::ArrowSize => &style.arrow_size,
        Metric::Margin => &style.margin,
        Metric::LineHeight => &style.line_height,
    }
}

fn metric_mut(style: &mut Style, metric: Metric) -> &mut Option<f32> {
    match metric {
        Metric::Zoom => &mut style.zoom,
        Metric::LabelSize => &mut style.label_size,
        Metric::TitleSize => &mut style.title_size,
        Metric::DetailSize => &mut style.detail_size,
        Metric::NodePadding => &mut style.node_padding,
        Metric::NodeMinWidth => &mut style.node_min_width,
        Metric::NodeMinHeight => &mut style.node_min_height,
        Metric::RankGap => &mut style.rank_gap,
        Metric::SiblingGap => &mut style.sibling_gap,
        Metric::CornerRadius => &mut style.corner_radius,
        Metric::BorderWidth => &mut style.border_width,
        Metric::EdgeWidth => &mut style.edge_width,
        Metric::ArrowSize => &mut style.arrow_size,
        Metric::Margin => &mut style.margin,
        Metric::LineHeight => &mut style.line_height,
    }
}

fn base_of(palette: &Palette, base: Base) -> &Option<String> {
    match base {
        Base::Background => &palette.background,
        Base::Surface => &palette.surface,
        Base::SurfaceAlt => &palette.surface_alt,
        Base::Border => &palette.border,
        Base::Text => &palette.text,
        Base::TextMuted => &palette.text_muted,
        Base::Edge => &palette.edge,
        Base::GroupFill => &palette.group_fill,
        Base::GroupBorder => &palette.group_border,
    }
}

fn base_mut(palette: &mut Palette, base: Base) -> &mut Option<String> {
    match base {
        Base::Background => &mut palette.background,
        Base::Surface => &mut palette.surface,
        Base::SurfaceAlt => &mut palette.surface_alt,
        Base::Border => &mut palette.border,
        Base::Text => &mut palette.text,
        Base::TextMuted => &mut palette.text_muted,
        Base::Edge => &mut palette.edge,
        Base::GroupFill => &mut palette.group_fill,
        Base::GroupBorder => &mut palette.group_border,
    }
}

fn accent_of(palette: &Palette, slot: Slot) -> &AccentOverride {
    match slot {
        Slot::Primary => &palette.primary,
        Slot::Success => &palette.success,
        Slot::Warning => &palette.warning,
        Slot::Danger => &palette.danger,
        Slot::Info => &palette.info,
        Slot::Muted => &palette.muted,
    }
}

fn accent_mut(palette: &mut Palette, slot: Slot) -> &mut AccentOverride {
    match slot {
        Slot::Primary => &mut palette.primary,
        Slot::Success => &mut palette.success,
        Slot::Warning => &mut palette.warning,
        Slot::Danger => &mut palette.danger,
        Slot::Info => &mut palette.info,
        Slot::Muted => &mut palette.muted,
    }
}

fn part_of(accent: &AccentOverride, part: Part) -> &Option<String> {
    match part {
        Part::Fill => &accent.fill,
        Part::Border => &accent.border,
        Part::Strong => &accent.strong,
        Part::Text => &accent.text,
    }
}

fn part_mut(accent: &mut AccentOverride, part: Part) -> &mut Option<String> {
    match part {
        Part::Fill => &mut accent.fill,
        Part::Border => &mut accent.border,
        Part::Strong => &mut accent.strong,
        Part::Text => &mut accent.text,
    }
}
