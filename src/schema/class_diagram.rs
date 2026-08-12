use crate::schema::common::{Accent, Direction, Style, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClassDiagram {
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
    pub classes: Vec<Class>,
    #[serde(default)]
    pub relations: Vec<ClassRelation>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Class {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stereotype: Option<String>,
    #[serde(default)]
    pub fields: Vec<Member>,
    #[serde(default)]
    pub methods: Vec<Member>,
    #[serde(default)]
    pub accent: Accent,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Member {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub visibility: Visibility,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub is_abstract: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClassRelation {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub kind: RelationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_cardinality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_cardinality: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    #[default]
    Association,
    Inheritance,
    Realization,
    Composition,
    Aggregation,
    Dependency,
}
