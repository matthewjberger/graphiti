#![allow(dead_code)]

use crate::description::Error;
use lazy_static::lazy_static;
use legion::World;
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use std::sync::RwLock;

type Result<T, E = Error> = std::result::Result<T, E>;

lazy_static! {
    pub static ref COMPONENT_REGISTRY: RwLock<legion::Registry<String>> =
        RwLock::new(legion::Registry::default());
    pub static ref ENTITY_SERIALIZER: legion::serialize::Canon =
        legion::serialize::Canon::default();
}

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
