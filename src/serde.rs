#![allow(dead_code)]

use crate::Error;
use legion::World;
use once_cell::sync::Lazy;
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use std::sync::RwLock;

type Result<T, E = Error> = std::result::Result<T, E>;

pub static COMPONENT_REGISTRY: Lazy<RwLock<legion::Registry<String>>> = Lazy::new(Default::default);
pub static ENTITY_SERIALIZER: Lazy<legion::serialize::Canon> = Lazy::new(Default::default);

pub fn register_component<T: legion::storage::Component + Serialize + for<'de> Deserialize<'de>>(
    key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = COMPONENT_REGISTRY
        .write()
        .map_err(|_| Error::AccessComponentRegistry)?;
    registry.register::<T>(key.to_string());
    Ok(())
}

pub fn serialize_ecs<S>(ecs: &World, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let registry = COMPONENT_REGISTRY
        .read()
        .expect("Failed to get the component registry lock!");
    ecs.as_serializable(legion::any(), &*registry, &*ENTITY_SERIALIZER)
        .serialize(serializer)
}

pub fn deserialize_ecs<'de, D>(deserializer: D) -> Result<World, D::Error>
where
    D: serde::Deserializer<'de>,
{
    COMPONENT_REGISTRY
        .read()
        .expect("Failed to get the component registry lock!")
        .as_deserialize(&*ENTITY_SERIALIZER)
        .deserialize(deserializer)
}

// TODO: Use this description attribute on the legion ecs world
// #[serde(serialize_with = "serialize_ecs", deserialize_with = "deserialize_ecs")]
