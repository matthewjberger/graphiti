use legion::{Entity, World};
use once_cell::sync::Lazy;
use petgraph::graph::DiGraph;
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use snafu::{OptionExt, Snafu};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Node '{}' not found", name))]
    NodeNotFound { name: String },

    #[snafu(display("Invalid parameters"))]
    InvalidParameters,

    #[snafu(display("Invalid edge name"))]
    InvalidEdgeName,

    #[snafu(display("Failed to access component registry"))]
    AccessComponentRegistry,

    #[snafu(display("Failed to access ECS world"))]
    AccessWorld,
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub static COMPONENT_REGISTRY: Lazy<RwLock<legion::Registry<String>>> = Lazy::new(Default::default);
pub static ENTITY_SERIALIZER: Lazy<legion::serialize::Canon> = Lazy::new(Default::default);

pub fn register_component<T: legion::storage::Component + Serialize + for<'de> Deserialize<'de>>(
    key: &str,
) -> Result<()> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Description {
    #[serde(serialize_with = "serialize_ecs", deserialize_with = "deserialize_ecs")]
    pub data: World,
    pub graphs: HashMap<String, DiGraph<Entity, String>>,
}

pub struct DescriptionBuilder {
    world: World,
    node_indices: HashMap<String, Entity>,
    graphs: GraphContainer,
}

impl DescriptionBuilder {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            node_indices: HashMap::new(),
            graphs: GraphContainer::new(),
        }
    }

    pub fn add_node(&mut self, name: String, value: String) -> Result<&mut Self> {
        if name.is_empty() || value.is_empty() {
            return Err(Error::InvalidParameters);
        }
        let entity = self.world.push((value,));
        self.node_indices.insert(name, entity);
        Ok(self)
    }

    pub fn add_edge(
        &mut self,
        edge_name: String,
        source: String,
        targets: Vec<String>,
    ) -> Result<&mut Self> {
        if edge_name.is_empty() {
            return Err(Error::InvalidEdgeName);
        }
        self.graphs
            .add_edge(edge_name, source, &self.node_indices, targets)?;
        Ok(self)
    }

    pub fn build(self) -> Description {
        Description {
            data: self.world,
            graphs: self.graphs.graphs,
        }
    }
}

struct GraphContainer {
    graphs: HashMap<String, DiGraph<Entity, String>>,
}

impl GraphContainer {
    fn new() -> Self {
        GraphContainer {
            graphs: HashMap::new(),
        }
    }

    fn add_edge(
        &mut self,
        edge_name: String,
        source: String,
        node_indices: &HashMap<String, Entity>,
        targets: Vec<String>,
    ) -> Result<()> {
        let graph = self
            .graphs
            .entry(edge_name.clone())
            .or_insert_with(DiGraph::new);
        let source_entity = node_indices
            .get(&source)
            .context(NodeNotFoundSnafu { name: source })?;
        let source_index = graph
            .node_indices()
            .find(|i| graph[*i] == *source_entity)
            .unwrap_or_else(|| graph.add_node(*source_entity));

        for target in targets {
            let target_entity = node_indices
                .get(&target)
                .context(NodeNotFoundSnafu { name: target })?;
            let target_index = graph
                .node_indices()
                .find(|i| graph[*i] == *target_entity)
                .unwrap_or_else(|| graph.add_node(*target_entity));
            graph.add_edge(source_index, target_index, edge_name.clone());
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! describe {
    (
        nodes: {
            $($node_name:ident : $node_value:expr => $comp:ty),* $(,)*
        },
        edges: {
            $($edge_name:literal : {
                $($source:ident : [$($target:ident),* $(,)*]),* $(,)*
            }),*
        }
    ) => {
        {
            $(
                register_component::<$comp>(stringify!($comp))?; // Automatically register the component
            )*

            let mut builder = $crate::DescriptionBuilder::new();
            $(
                builder.add_node(stringify!($node_name).to_string(), $node_value.to_string())?;
            )*
            $(
                $(
                    builder.add_edge($edge_name.to_string(), stringify!($source).to_string(), vec![$(stringify!($target).to_string()),*])?;
                )*
            )*
            builder.build()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_component() -> Result<()> {
        register_component::<i32>("i32")
    }

    #[test]
    fn test_description_builder() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        builder.add_node("node1".to_string(), "value1".to_string())?;
        builder.add_node("node2".to_string(), "value2".to_string())?;
        builder.add_edge(
            "edge1".to_string(),
            "node1".to_string(),
            vec!["node2".to_string()],
        )?;
        let description = builder.build();
        assert_eq!(description.graphs.contains_key("edge1"), true);
        Ok(())
    }

    #[test]
    fn test_add_node_with_empty_name() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        let result = builder.add_node("".to_string(), "value".to_string());
        assert_eq!(result.is_err(), true);
        Ok(())
    }

    #[test]
    fn test_add_edge_with_empty_name() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        let result = builder.add_edge(
            "".to_string(),
            "source".to_string(),
            vec!["target".to_string()],
        );
        assert_eq!(result.is_err(), true);
        Ok(())
    }

    #[test]
    fn test_add_edge_with_missing_node() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        builder.add_node("source".to_string(), "value".to_string())?;
        let result = builder.add_edge(
            "edge1".to_string(),
            "source".to_string(),
            vec!["missing".to_string()],
        );
        assert_eq!(result.is_err(), true);
        Ok(())
    }

    #[test]
    fn test_dsl_macro() -> Result<()> {
        let description = describe! {
            nodes: {
                node1: "value1" => String,
                node2: "value2" => String
            },
            edges: {
                "edge_name": {
                    node1: [node2]
                }
            }
        };
        assert!(description.graphs.contains_key("edge_name"));
        Ok(())
    }
}
