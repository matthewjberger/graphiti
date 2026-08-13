use graphiti::schema::class_diagram::RelationKind;
use graphiti::schema::common::{
    Accent, ArrowHead, Direction, EdgeRouting, LineStyle, NodeShape, Visibility,
};
use graphiti::schema::entity_relationship::{Cardinality, KeyKind};
use graphiti::schema::sequence::{FragmentKind, MessageKind, NotePlacement, ParticipantKind};
use graphiti::schema::state_diagram::StateKind;

pub const KINDS: &[(&str, &str)] = &[
    ("Flowchart", "flowchart"),
    ("Sequence", "sequence"),
    ("Class", "class"),
    ("State", "state"),
    ("Entity relationship", "entity_relationship"),
];

pub const BASE_THEMES: &[(&str, &str)] = &[("Light", "light"), ("Dark", "dark"), ("Mono", "mono")];

pub const THEMES: &[(&str, Option<&str>)] = &[
    ("Inherit", None),
    ("Light", Some("light")),
    ("Dark", Some("dark")),
    ("Mono", Some("mono")),
];

pub const DIRECTIONS: &[(&str, Direction)] = &[
    ("Down", Direction::Down),
    ("Up", Direction::Up),
    ("Right", Direction::Right),
    ("Left", Direction::Left),
];

pub const ROUTINGS: &[(&str, EdgeRouting)] = &[
    ("Orthogonal", EdgeRouting::Orthogonal),
    ("Curved", EdgeRouting::Curved),
    ("Straight", EdgeRouting::Straight),
];

pub const SHAPES: &[(&str, NodeShape)] = &[
    ("Rectangle", NodeShape::Rectangle),
    ("Rounded", NodeShape::Rounded),
    ("Stadium", NodeShape::Stadium),
    ("Circle", NodeShape::Circle),
    ("Diamond", NodeShape::Diamond),
    ("Hexagon", NodeShape::Hexagon),
    ("Parallelogram", NodeShape::Parallelogram),
    ("Cylinder", NodeShape::Cylinder),
    ("Subroutine", NodeShape::Subroutine),
    ("Note", NodeShape::Note),
];

pub const ACCENTS: &[(&str, Accent)] = &[
    ("Neutral", Accent::Neutral),
    ("Primary", Accent::Primary),
    ("Success", Accent::Success),
    ("Warning", Accent::Warning),
    ("Danger", Accent::Danger),
    ("Info", Accent::Info),
    ("Muted", Accent::Muted),
];

pub const LINE_STYLES: &[(&str, LineStyle)] = &[
    ("Solid", LineStyle::Solid),
    ("Dashed", LineStyle::Dashed),
    ("Dotted", LineStyle::Dotted),
    ("Thick", LineStyle::Thick),
];

pub const ARROWS: &[(&str, ArrowHead)] = &[
    ("None", ArrowHead::None),
    ("Arrow", ArrowHead::Arrow),
    ("Open", ArrowHead::Open),
    ("Hollow triangle", ArrowHead::HollowTriangle),
    ("Diamond", ArrowHead::Diamond),
    ("Hollow diamond", ArrowHead::HollowDiamond),
    ("Circle", ArrowHead::Circle),
    ("Hollow circle", ArrowHead::HollowCircle),
    ("Bar", ArrowHead::Bar),
    ("Crow's foot", ArrowHead::CrowsFoot),
    ("Crow's foot one", ArrowHead::CrowsFootOne),
    ("Crow's foot zero or one", ArrowHead::CrowsFootZeroOrOne),
    ("Crow's foot zero or many", ArrowHead::CrowsFootZeroOrMany),
    ("Crow's foot one or many", ArrowHead::CrowsFootOneOrMany),
];

pub const VISIBILITIES: &[(&str, Visibility)] = &[
    ("Public", Visibility::Public),
    ("Private", Visibility::Private),
    ("Protected", Visibility::Protected),
    ("Package", Visibility::Package),
];

pub const RELATION_KINDS: &[(&str, RelationKind)] = &[
    ("Association", RelationKind::Association),
    ("Inheritance", RelationKind::Inheritance),
    ("Realization", RelationKind::Realization),
    ("Composition", RelationKind::Composition),
    ("Aggregation", RelationKind::Aggregation),
    ("Dependency", RelationKind::Dependency),
];

pub const STATE_KINDS: &[(&str, StateKind)] = &[
    ("Simple", StateKind::Simple),
    ("Start", StateKind::Start),
    ("End", StateKind::End),
    ("Choice", StateKind::Choice),
    ("Fork", StateKind::Fork),
    ("Join", StateKind::Join),
];

pub const PARTICIPANT_KINDS: &[(&str, ParticipantKind)] = &[
    ("Participant", ParticipantKind::Participant),
    ("Actor", ParticipantKind::Actor),
    ("Database", ParticipantKind::Database),
    ("Boundary", ParticipantKind::Boundary),
];

pub const MESSAGE_KINDS: &[(&str, MessageKind)] = &[
    ("Sync", MessageKind::Sync),
    ("Async", MessageKind::Async),
    ("Reply", MessageKind::Reply),
    ("Create", MessageKind::Create),
    ("Destroy", MessageKind::Destroy),
];

pub const NOTE_PLACEMENTS: &[(&str, NotePlacement)] = &[
    ("Over", NotePlacement::Over),
    ("Left", NotePlacement::Left),
    ("Right", NotePlacement::Right),
];

pub const FRAGMENT_KINDS: &[(&str, FragmentKind)] = &[
    ("Loop", FragmentKind::Loop),
    ("Alt", FragmentKind::Alt),
    ("Opt", FragmentKind::Opt),
    ("Par", FragmentKind::Par),
    ("Critical", FragmentKind::Critical),
    ("Break", FragmentKind::Break),
];

pub const KEY_KINDS: &[(&str, KeyKind)] = &[
    ("None", KeyKind::None),
    ("Primary", KeyKind::Primary),
    ("Foreign", KeyKind::Foreign),
    ("Unique", KeyKind::Unique),
];

pub const CARDINALITIES: &[(&str, Cardinality)] = &[
    ("Exactly one", Cardinality::ExactlyOne),
    ("Zero or one", Cardinality::ZeroOrOne),
    ("Zero or many", Cardinality::ZeroOrMany),
    ("One or many", Cardinality::OneOrMany),
];

pub const STEP_KINDS: &[(&str, &str)] = &[
    ("Message", "message"),
    ("Note", "note"),
    ("Fragment", "fragment"),
    ("Divider", "divider"),
];

pub fn theme_name(value: Option<&str>) -> Option<&'static str> {
    THEMES
        .iter()
        .find_map(|(_, candidate)| candidate.filter(|name| Some(*name) == value))
}
