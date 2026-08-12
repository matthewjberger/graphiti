use crate::schema::common::Accent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Sequence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub participants: Vec<Participant>,
    #[serde(default)]
    pub steps: Vec<SequenceStep>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub kind: ParticipantKind,
    #[serde(default)]
    pub accent: Accent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantKind {
    #[default]
    Participant,
    Actor,
    Database,
    Boundary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "step", rename_all = "snake_case")]
pub enum SequenceStep {
    Message(Message),
    Note(Note),
    Fragment(Fragment),
    Divider(Divider),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub kind: MessageKind,
    #[serde(default)]
    pub accent: Accent,
    #[serde(default)]
    pub activate: bool,
    #[serde(default)]
    pub deactivate: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    #[default]
    Sync,
    Async,
    Reply,
    Create,
    Destroy,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Note {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub over: Vec<String>,
    #[serde(default)]
    pub placement: NotePlacement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotePlacement {
    #[default]
    Over,
    Left,
    Right,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Fragment {
    #[serde(default)]
    pub kind: FragmentKind,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub steps: Vec<SequenceStep>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<FragmentBranch>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FragmentBranch {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub steps: Vec<SequenceStep>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentKind {
    #[default]
    Loop,
    Alt,
    Opt,
    Par,
    Critical,
    Break,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Divider {
    #[serde(default)]
    pub label: String,
}
