use crate::schema::common::{Accent, Direction, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EntityRelationship {
    #[serde(
        default,
        skip_serializing_if = "crate::schema::common::style_is_default"
    )]
    pub style: Style,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub direction: Direction,
    #[serde(default)]
    pub entities: Vec<Entity>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    #[serde(default)]
    pub accent: Accent,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub key: KeyKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyKind {
    #[default]
    None,
    Primary,
    Foreign,
    Unique,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub from_cardinality: Cardinality,
    #[serde(default)]
    pub to_cardinality: Cardinality,
    #[serde(default = "identifying_default")]
    pub identifying: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    ExactlyOne,
    ZeroOrOne,
    #[default]
    ZeroOrMany,
    OneOrMany,
}

fn identifying_default() -> bool {
    true
}
