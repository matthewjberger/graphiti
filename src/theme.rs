use crate::schema::Accent;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rgba {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct AccentColors {
    pub fill: Rgba,
    pub border: Rgba,
    pub strong: Rgba,
    pub text: Rgba,
}

#[derive(Clone, Copy, Debug)]
pub struct Metrics {
    pub title_size: f32,
    pub label_size: f32,
    pub detail_size: f32,
    pub member_size: f32,
    pub node_padding_x: f32,
    pub node_padding_y: f32,
    pub node_min_width: f32,
    pub node_min_height: f32,
    pub rank_gap: f32,
    pub sibling_gap: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub edge_width: f32,
    pub arrow_size: f32,
    pub margin: f32,
    pub group_padding: f32,
    pub line_height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Rgba,
    pub surface: Rgba,
    pub surface_alt: Rgba,
    pub border: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub edge: Rgba,
    pub edge_label_background: Rgba,
    pub group_fill: Rgba,
    pub group_border: Rgba,
    pub primary: AccentColors,
    pub success: AccentColors,
    pub warning: AccentColors,
    pub danger: AccentColors,
    pub info: AccentColors,
    pub muted: AccentColors,
    pub metrics: Metrics,
}

pub fn srgb_channel_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

pub fn rgba_from_hex(hex: u32, alpha: f32) -> Rgba {
    let red = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let green = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let blue = (hex & 0xFF) as f32 / 255.0;
    Rgba {
        red: srgb_channel_to_linear(red),
        green: srgb_channel_to_linear(green),
        blue: srgb_channel_to_linear(blue),
        alpha,
    }
}

pub fn rgba_array(color: Rgba) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha]
}

pub fn default_metrics() -> Metrics {
    Metrics {
        title_size: 30.0,
        label_size: 17.0,
        detail_size: 13.0,
        member_size: 14.0,
        node_padding_x: 22.0,
        node_padding_y: 16.0,
        node_min_width: 104.0,
        node_min_height: 52.0,
        rank_gap: 84.0,
        sibling_gap: 44.0,
        corner_radius: 8.0,
        border_width: 1.6,
        edge_width: 1.8,
        arrow_size: 11.0,
        margin: 48.0,
        group_padding: 30.0,
        line_height: 1.35,
    }
}

pub fn theme_light() -> Theme {
    Theme {
        background: rgba_from_hex(0xFCFCFD, 1.0),
        surface: rgba_from_hex(0xFFFFFF, 1.0),
        surface_alt: rgba_from_hex(0xF1F4F9, 1.0),
        border: rgba_from_hex(0xC9D2E0, 1.0),
        text: rgba_from_hex(0x1B2330, 1.0),
        text_muted: rgba_from_hex(0x64748B, 1.0),
        edge: rgba_from_hex(0x8794A8, 1.0),
        edge_label_background: rgba_from_hex(0xFCFCFD, 1.0),
        group_fill: rgba_from_hex(0xF4F6FA, 1.0),
        group_border: rgba_from_hex(0xD5DDE9, 1.0),
        primary: AccentColors {
            fill: rgba_from_hex(0xEAF1FE, 1.0),
            border: rgba_from_hex(0x6C9BF5, 1.0),
            strong: rgba_from_hex(0x2563EB, 1.0),
            text: rgba_from_hex(0x14305F, 1.0),
        },
        success: AccentColors {
            fill: rgba_from_hex(0xE7F7EF, 1.0),
            border: rgba_from_hex(0x5DC08C, 1.0),
            strong: rgba_from_hex(0x159A5C, 1.0),
            text: rgba_from_hex(0x0C3B26, 1.0),
        },
        warning: AccentColors {
            fill: rgba_from_hex(0xFDF3E2, 1.0),
            border: rgba_from_hex(0xE0A94A, 1.0),
            strong: rgba_from_hex(0xB97709, 1.0),
            text: rgba_from_hex(0x4B3208, 1.0),
        },
        danger: AccentColors {
            fill: rgba_from_hex(0xFDECEC, 1.0),
            border: rgba_from_hex(0xE58787, 1.0),
            strong: rgba_from_hex(0xD03B3B, 1.0),
            text: rgba_from_hex(0x521818, 1.0),
        },
        info: AccentColors {
            fill: rgba_from_hex(0xE8F5F9, 1.0),
            border: rgba_from_hex(0x63B4CE, 1.0),
            strong: rgba_from_hex(0x0E7C99, 1.0),
            text: rgba_from_hex(0x0C333F, 1.0),
        },
        muted: AccentColors {
            fill: rgba_from_hex(0xF3F5F8, 1.0),
            border: rgba_from_hex(0xC2CBD8, 1.0),
            strong: rgba_from_hex(0x76839A, 1.0),
            text: rgba_from_hex(0x39424F, 1.0),
        },
        metrics: default_metrics(),
    }
}

pub fn theme_dark() -> Theme {
    Theme {
        background: rgba_from_hex(0x14171D, 1.0),
        surface: rgba_from_hex(0x1D2129, 1.0),
        surface_alt: rgba_from_hex(0x252B35, 1.0),
        border: rgba_from_hex(0x39404D, 1.0),
        text: rgba_from_hex(0xE9ECF2, 1.0),
        text_muted: rgba_from_hex(0x9AA5B6, 1.0),
        edge: rgba_from_hex(0x76839A, 1.0),
        edge_label_background: rgba_from_hex(0x14171D, 1.0),
        group_fill: rgba_from_hex(0x1A1E25, 1.0),
        group_border: rgba_from_hex(0x333A46, 1.0),
        primary: AccentColors {
            fill: rgba_from_hex(0x1B2A47, 1.0),
            border: rgba_from_hex(0x3E6DC4, 1.0),
            strong: rgba_from_hex(0x6BA0FF, 1.0),
            text: rgba_from_hex(0xD3E2FF, 1.0),
        },
        success: AccentColors {
            fill: rgba_from_hex(0x16301F, 1.0),
            border: rgba_from_hex(0x2E7A4F, 1.0),
            strong: rgba_from_hex(0x54C98A, 1.0),
            text: rgba_from_hex(0xCDF0DC, 1.0),
        },
        warning: AccentColors {
            fill: rgba_from_hex(0x33260F, 1.0),
            border: rgba_from_hex(0x8A6520, 1.0),
            strong: rgba_from_hex(0xE0A94A, 1.0),
            text: rgba_from_hex(0xF7E7C4, 1.0),
        },
        danger: AccentColors {
            fill: rgba_from_hex(0x361A1A, 1.0),
            border: rgba_from_hex(0x8E3A3A, 1.0),
            strong: rgba_from_hex(0xEA7A7A, 1.0),
            text: rgba_from_hex(0xF8D8D8, 1.0),
        },
        info: AccentColors {
            fill: rgba_from_hex(0x142B33, 1.0),
            border: rgba_from_hex(0x2A6E85, 1.0),
            strong: rgba_from_hex(0x5CBCD8, 1.0),
            text: rgba_from_hex(0xCCEBF4, 1.0),
        },
        muted: AccentColors {
            fill: rgba_from_hex(0x21262E, 1.0),
            border: rgba_from_hex(0x3C4553, 1.0),
            strong: rgba_from_hex(0x8B97A8, 1.0),
            text: rgba_from_hex(0xC3CBD8, 1.0),
        },
        metrics: default_metrics(),
    }
}

pub fn theme_mono() -> Theme {
    let ink = rgba_from_hex(0x14161A, 1.0);
    let muted = rgba_from_hex(0x6E747D, 1.0);
    let plain = AccentColors {
        fill: rgba_from_hex(0xF2F3F5, 1.0),
        border: rgba_from_hex(0x9CA3AC, 1.0),
        strong: ink,
        text: ink,
    };
    Theme {
        background: rgba_from_hex(0xFFFFFF, 1.0),
        surface: rgba_from_hex(0xFFFFFF, 1.0),
        surface_alt: rgba_from_hex(0xF2F3F5, 1.0),
        border: rgba_from_hex(0x8B9199, 1.0),
        text: ink,
        text_muted: muted,
        edge: rgba_from_hex(0x5B6169, 1.0),
        edge_label_background: rgba_from_hex(0xFFFFFF, 1.0),
        group_fill: rgba_from_hex(0xF7F8F9, 1.0),
        group_border: rgba_from_hex(0xB6BBC2, 1.0),
        primary: plain,
        success: plain,
        warning: plain,
        danger: plain,
        info: plain,
        muted: AccentColors {
            fill: rgba_from_hex(0xF7F8F9, 1.0),
            border: rgba_from_hex(0xB6BBC2, 1.0),
            strong: muted,
            text: muted,
        },
        metrics: default_metrics(),
    }
}

pub fn theme_by_name(name: &str) -> Option<Theme> {
    match name {
        "light" => Some(theme_light()),
        "dark" => Some(theme_dark()),
        "mono" => Some(theme_mono()),
        _ => None,
    }
}

pub fn theme_names() -> &'static [&'static str] {
    &["light", "dark", "mono"]
}

pub fn parse_hex(value: &str) -> Option<Rgba> {
    let digits = value.trim().trim_start_matches('#');
    match digits.len() {
        6 => u32::from_str_radix(digits, 16)
            .ok()
            .map(|hex| rgba_from_hex(hex, 1.0)),
        8 => u32::from_str_radix(&digits[0..6], 16).ok().and_then(|hex| {
            u8::from_str_radix(&digits[6..8], 16)
                .ok()
                .map(|alpha| rgba_from_hex(hex, alpha as f32 / 255.0))
        }),
        _ => None,
    }
}

pub fn apply_style(base: &Theme, style: &crate::schema::common::Style) -> Theme {
    let mut theme = style
        .theme
        .as_deref()
        .and_then(theme_by_name)
        .unwrap_or(*base);

    let palette = &style.palette;
    override_color(&mut theme.background, palette.background.as_deref());
    override_color(
        &mut theme.edge_label_background,
        palette.background.as_deref(),
    );
    override_color(&mut theme.surface, palette.surface.as_deref());
    override_color(&mut theme.surface_alt, palette.surface_alt.as_deref());
    override_color(&mut theme.border, palette.border.as_deref());
    override_color(&mut theme.text, palette.text.as_deref());
    override_color(&mut theme.text_muted, palette.text_muted.as_deref());
    override_color(&mut theme.edge, palette.edge.as_deref());
    override_color(&mut theme.group_fill, palette.group_fill.as_deref());
    override_color(&mut theme.group_border, palette.group_border.as_deref());
    override_accent(&mut theme.primary, &palette.primary);
    override_accent(&mut theme.success, &palette.success);
    override_accent(&mut theme.warning, &palette.warning);
    override_accent(&mut theme.danger, &palette.danger);
    override_accent(&mut theme.info, &palette.info);
    override_accent(&mut theme.muted, &palette.muted);

    if style.compact {
        theme.metrics = compact_metrics(theme.metrics);
    }

    let metrics = &mut theme.metrics;
    if let Some(size) = style.label_size {
        let ratio = size / metrics.label_size.max(0.01);
        metrics.label_size = size;
        metrics.member_size *= ratio;
        metrics.detail_size *= ratio;
    }
    override_value(&mut metrics.title_size, style.title_size);
    override_value(&mut metrics.detail_size, style.detail_size);
    if let Some(padding) = style.node_padding {
        metrics.node_padding_x = padding;
        metrics.node_padding_y = padding * 0.72;
    }
    override_value(&mut metrics.node_min_width, style.node_min_width);
    override_value(&mut metrics.node_min_height, style.node_min_height);
    override_value(&mut metrics.rank_gap, style.rank_gap);
    override_value(&mut metrics.sibling_gap, style.sibling_gap);
    override_value(&mut metrics.corner_radius, style.corner_radius);
    override_value(&mut metrics.border_width, style.border_width);
    override_value(&mut metrics.edge_width, style.edge_width);
    override_value(&mut metrics.arrow_size, style.arrow_size);
    override_value(&mut metrics.margin, style.margin);
    override_value(&mut metrics.line_height, style.line_height);

    if let Some(zoom) = style.zoom {
        theme.metrics = scale_metrics(theme.metrics, zoom.clamp(0.25, 4.0));
    }
    theme
}

pub fn scale_metrics(metrics: Metrics, factor: f32) -> Metrics {
    Metrics {
        title_size: metrics.title_size * factor,
        label_size: metrics.label_size * factor,
        detail_size: metrics.detail_size * factor,
        member_size: metrics.member_size * factor,
        node_padding_x: metrics.node_padding_x * factor,
        node_padding_y: metrics.node_padding_y * factor,
        node_min_width: metrics.node_min_width * factor,
        node_min_height: metrics.node_min_height * factor,
        rank_gap: metrics.rank_gap * factor,
        sibling_gap: metrics.sibling_gap * factor,
        corner_radius: metrics.corner_radius * factor,
        border_width: metrics.border_width * factor,
        edge_width: metrics.edge_width * factor,
        arrow_size: metrics.arrow_size * factor,
        margin: metrics.margin * factor,
        group_padding: metrics.group_padding * factor,
        line_height: metrics.line_height,
    }
}

fn compact_metrics(metrics: Metrics) -> Metrics {
    Metrics {
        node_padding_x: metrics.node_padding_x * 0.7,
        node_padding_y: metrics.node_padding_y * 0.7,
        node_min_width: metrics.node_min_width * 0.85,
        node_min_height: metrics.node_min_height * 0.8,
        rank_gap: metrics.rank_gap * 0.6,
        sibling_gap: metrics.sibling_gap * 0.65,
        margin: metrics.margin * 0.6,
        group_padding: metrics.group_padding * 0.7,
        ..metrics
    }
}

fn override_value(target: &mut f32, value: Option<f32>) {
    if let Some(value) = value
        && value.is_finite()
        && value > 0.0
    {
        *target = value;
    }
}

fn override_color(target: &mut Rgba, value: Option<&str>) {
    if let Some(color) = value.and_then(parse_hex) {
        *target = color;
    }
}

fn override_accent(target: &mut AccentColors, value: &crate::schema::common::AccentOverride) {
    override_color(&mut target.fill, value.fill.as_deref());
    override_color(&mut target.border, value.border.as_deref());
    override_color(&mut target.strong, value.strong.as_deref());
    override_color(&mut target.text, value.text.as_deref());
}

pub fn accent_colors(theme: &Theme, accent: Accent) -> AccentColors {
    match accent {
        Accent::Neutral => AccentColors {
            fill: theme.surface,
            border: theme.border,
            strong: theme.text_muted,
            text: theme.text,
        },
        Accent::Primary => theme.primary,
        Accent::Success => theme.success,
        Accent::Warning => theme.warning,
        Accent::Danger => theme.danger,
        Accent::Info => theme.info,
        Accent::Muted => theme.muted,
    }
}
