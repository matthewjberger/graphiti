mod anymap;
mod description;
mod serde;

pub use self::{
    anymap::AnyMap,
    description::{Description, DescriptionBuilder, Error},
    serde::{deserialize_ecs, register_component, serialize_ecs},
};
