use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Down,
    Up,
    Right,
    Left,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeShape {
    #[default]
    Rectangle,
    Rounded,
    Stadium,
    Circle,
    Diamond,
    Hexagon,
    Parallelogram,
    Cylinder,
    Subroutine,
    Note,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    Thick,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrowHead {
    None,
    #[default]
    Arrow,
    Open,
    HollowTriangle,
    Diamond,
    HollowDiamond,
    Circle,
    HollowCircle,
    Bar,
    CrowsFoot,
    CrowsFootOne,
    CrowsFootZeroOrOne,
    CrowsFootZeroOrMany,
    CrowsFootOneOrMany,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Accent {
    #[default]
    Neutral,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
    Muted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Protected,
    Package,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeRouting {
    #[default]
    Orthogonal,
    Curved,
    Straight,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Style {
    pub theme: Option<String>,
    pub zoom: Option<f32>,
    pub compact: bool,
    pub monospace: bool,
    pub label_size: Option<f32>,
    pub title_size: Option<f32>,
    pub detail_size: Option<f32>,
    pub node_padding: Option<f32>,
    pub node_min_width: Option<f32>,
    pub node_min_height: Option<f32>,
    pub rank_gap: Option<f32>,
    pub sibling_gap: Option<f32>,
    pub corner_radius: Option<f32>,
    pub border_width: Option<f32>,
    pub edge_width: Option<f32>,
    pub arrow_size: Option<f32>,
    pub margin: Option<f32>,
    pub line_height: Option<f32>,
    pub palette: Palette,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Palette {
    pub background: Option<String>,
    pub surface: Option<String>,
    pub surface_alt: Option<String>,
    pub border: Option<String>,
    pub text: Option<String>,
    pub text_muted: Option<String>,
    pub edge: Option<String>,
    pub group_fill: Option<String>,
    pub group_border: Option<String>,
    pub primary: AccentOverride,
    pub success: AccentOverride,
    pub warning: AccentOverride,
    pub danger: AccentOverride,
    pub info: AccentOverride,
    pub muted: AccentOverride,
}

pub fn style_is_default(style: &Style) -> bool {
    style == &Style::default()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AccentOverride {
    pub fill: Option<String>,
    pub border: Option<String>,
    pub strong: Option<String>,
    pub text: Option<String>,
}
