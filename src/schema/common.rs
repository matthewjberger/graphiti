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
