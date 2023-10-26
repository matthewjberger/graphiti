use crate::AnyMap;
use legion::{storage::IntoComponentSource, Entity, EntityStore, World};
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Node '{name}' not found"))]
    NodeNotFound { name: String },

    #[snafu(display("Invalid parameters"))]
    InvalidParameters,

    #[snafu(display("Invalid edge name"))]
    InvalidEdgeName,

    #[snafu(display("Failed to access component registry"))]
    AccessComponentRegistry,
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Description {
    #[serde(
        serialize_with = "crate::serialize_ecs",
        deserialize_with = "crate::deserialize_ecs"
    )]
    pub data: World,
    pub node_name_to_entity: HashMap<String, Entity>,
    pub graphs: HashMap<String, DiGraph<Entity, String>>,
}

impl Description {
    pub fn get_component<T: legion::storage::Component>(&self, node_name: &str) -> Option<&T> {
        let entity = self.node_name_to_entity.get(node_name)?;
        self.data.entry_ref(*entity).ok()?.into_component().ok()
    }

    pub fn get_component_mut<T: legion::storage::Component>(
        &mut self,
        node_name: &str,
    ) -> Option<&mut T> {
        let entity = self.node_name_to_entity.get(node_name)?;
        self.data.entry_mut(*entity).ok()?.into_component_mut().ok()
    }

    pub fn outgoing_edges(&self, node_name: &str) -> Result<Vec<String>> {
        let entity = self
            .node_name_to_entity
            .get(node_name)
            .context(NodeNotFoundSnafu {
                name: node_name.to_string(),
            })?;
        let mut edges = Vec::new();
        for graph in self.graphs.values() {
            let node_index = graph.node_indices().find(|i| graph[*i] == *entity).unwrap();
            for edge in graph.edges_directed(node_index, petgraph::Direction::Outgoing) {
                edges.push(edge.weight().clone());
            }
        }
        Ok(edges)
    }

    pub fn incoming_edges(&self, node_name: &str) -> Result<Vec<String>> {
        let entity = self
            .node_name_to_entity
            .get(node_name)
            .context(NodeNotFoundSnafu {
                name: node_name.to_string(),
            })?;
        let mut edges = Vec::new();
        for graph in self.graphs.values() {
            let node_index = graph.node_indices().find(|i| graph[*i] == *entity).unwrap();
            for edge in graph.edges_directed(node_index, petgraph::Direction::Incoming) {
                edges.push(edge.weight().clone());
            }
        }
        Ok(edges)
    }

    pub fn connected_nodes(&self, node_name: &str) -> Result<Vec<String>> {
        let entity = self
            .node_name_to_entity
            .get(node_name)
            .context(NodeNotFoundSnafu {
                name: node_name.to_string(),
            })?;
        let mut nodes = Vec::new();
        for graph in self.graphs.values() {
            let node_index = graph.node_indices().find(|i| graph[*i] == *entity).unwrap();
            for neighbor_index in graph.neighbors(node_index) {
                if let Some(name) = self
                    .node_name_to_entity
                    .iter()
                    .find(|&(_, &e)| e == graph[neighbor_index])
                {
                    nodes.push(name.0.clone());
                }
            }
        }
        Ok(nodes)
    }

    pub fn has_direct_edge(&self, from_node: &str, to_node: &str) -> Result<bool> {
        let from_entity = self
            .node_name_to_entity
            .get(from_node)
            .context(NodeNotFoundSnafu {
                name: from_node.to_string(),
            })?;
        let to_entity = self
            .node_name_to_entity
            .get(to_node)
            .context(NodeNotFoundSnafu {
                name: to_node.to_string(),
            })?;
        for graph in self.graphs.values() {
            let from_index = graph
                .node_indices()
                .find(|i| graph[*i] == *from_entity)
                .unwrap();
            let to_index = graph
                .node_indices()
                .find(|i| graph[*i] == *to_entity)
                .unwrap();
            if graph.contains_edge(from_index, to_index) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub struct DescriptionBuilder {
    world: World,
    node_name_to_entity: HashMap<String, Entity>,
    graphs: GraphContainer,
    node_component_types: HashMap<String, AnyMap>,
}

impl DescriptionBuilder {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            node_name_to_entity: HashMap::new(),
            graphs: GraphContainer::new(),
            node_component_types: HashMap::new(),
        }
    }

    pub fn add_node<T: Clone + 'static>(&mut self, name: String, components: T) -> Result<&mut Self>
    where
        Option<T>: IntoComponentSource,
    {
        if name.is_empty() {
            return Err(Error::InvalidParameters);
        }

        // Get the AnyMap for the specific node, or create a new one
        let node_map = self
            .node_component_types
            .entry(name.clone())
            .or_insert_with(AnyMap::new);

        // Check if the component type is already added to this node
        if node_map.find::<T>().is_some() {
            return Err(Error::InvalidParameters); // Or a more descriptive error indicating duplicate component
        }

        // Add the component type to the node's AnyMap
        node_map.insert(components.clone());

        let entity = self.world.push(components);
        self.node_name_to_entity.insert(name, entity);
        Ok(self)
    }

    pub fn add_edge(
        &mut self,
        edge_name: &str,
        source_name: &str,
        target_names: Vec<&str>,
    ) -> Result<&mut Self> {
        if edge_name.is_empty() {
            return Err(Error::InvalidEdgeName);
        }

        self.graphs.add_edge(
            edge_name.to_string(),
            source_name.to_string(),
            &self.node_name_to_entity,
            target_names.iter().map(|s| s.to_string()).collect(),
        )?;
        Ok(self)
    }

    pub fn build(self) -> Description {
        Description {
            data: self.world,
            graphs: self.graphs.graphs,
            node_name_to_entity: self.node_name_to_entity,
        }
    }
}

#[derive(Debug)]
pub struct GraphContainer {
    graphs: HashMap<String, DiGraph<Entity, String>>,
}

impl GraphContainer {
    fn new() -> Self {
        GraphContainer {
            graphs: HashMap::new(),
        }
    }

    pub fn add_edge(
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
            $($node_name:ident : [$($comp_value:expr),* $(,)*]),* $(,)*
        },
        edges: {
            $($edge_name:literal : {
                $($source:ident : [$($target:ident),* $(,)*]),* $(,)*
        }),*
        }
    ) => {
        {
            let mut builder = $crate::DescriptionBuilder::new();
            $(
                builder.add_node(stringify!($node_name).to_string(), ($($comp_value,)*))?;
            )*
            $(
                $(
                    builder.add_edge($edge_name, stringify!($source), vec![$(stringify!($target)),*])?;
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
    fn test_description_builder() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        builder.add_node("node1".to_string(), ("value1",))?;
        builder.add_node("node2".to_string(), ("value2",))?;
        builder.add_edge("edge1", "node1", vec!["node2"])?;
        let description = builder.build();
        assert_eq!(description.graphs.contains_key("edge1"), true);
        Ok(())
    }

    #[test]
    fn test_add_node_with_empty_name() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        let result = builder.add_node("".to_string(), ("value",));
        assert_eq!(result.is_err(), true);
        Ok(())
    }

    #[test]
    fn test_add_edge_with_empty_name() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        let result = builder.add_edge("", "source", vec!["target"]);
        assert_eq!(result.is_err(), true);
        Ok(())
    }

    #[test]
    fn test_add_edge_with_missing_node() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        builder.add_node("source".to_string(), ("value",))?;
        let result = builder.add_edge("edge1", "source", vec!["missing"]);
        assert_eq!(result.is_err(), true);
        Ok(())
    }

    #[test]
    fn test_dsl_macro() -> Result<()> {
        let description = describe! {
            nodes: {
                node1: [
                    "value1".to_string(),
                    451,
                ],
                node2: [
                    "value1".to_string(),
                    32,
                    1.0_f32,
                ],
                node3: []
            },
            edges: {
                "edge_name":  {
                    node1: [node2]
                },
                "edge_name_2": {
                    node1: [node2, node3]
                }
            }
        };
        assert!(description.graphs.contains_key("edge_name"));
        Ok(())
    }

    #[test]
    fn test_outgoing_edges() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        builder.add_node("node1".to_string(), ("value1",))?;
        builder.add_node("node2".to_string(), ("value2",))?;
        builder.add_edge("edge1", "node1", vec!["node2"])?;
        let description = builder.build();

        let edges = description.outgoing_edges("node1")?;
        assert_eq!(edges, vec!["edge1"]);
        Ok(())
    }

    #[test]
    fn test_incoming_edges() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        builder.add_node("node1".to_string(), ("value1",))?;
        builder.add_node("node2".to_string(), ("value2",))?;
        builder.add_edge("edge1", "node2", vec!["node1"])?;
        let description = builder.build();

        let edges = description.incoming_edges("node1")?;
        assert_eq!(edges, vec!["edge1"]);
        Ok(())
    }

    #[test]
    fn test_connected_nodes() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        builder.add_node("node1".to_string(), ("value1",))?;
        builder.add_node("node2".to_string(), ("value2",))?;
        builder.add_node("node3".to_string(), ("value3",))?;
        builder.add_edge("edge1", "node1", vec!["node2"])?;
        builder.add_edge("edge2", "node1", vec!["node3"])?;
        let description = builder.build();

        let mut nodes = description.connected_nodes("node1")?;
        nodes.sort(); // Sort the nodes for consistent comparison
        let expected_nodes = vec!["node2", "node3"];
        assert_eq!(nodes, expected_nodes);
        Ok(())
    }

    #[test]
    fn test_has_direct_edge() -> Result<()> {
        let mut builder = DescriptionBuilder::new();
        builder.add_node("node1".to_string(), ("value1",))?;
        builder.add_node("node2".to_string(), ("value2",))?;
        builder.add_edge("edge1", "node1", vec!["node2"])?;
        let description = builder.build();

        assert_eq!(description.has_direct_edge("node1", "node2")?, true);
        assert_eq!(description.has_direct_edge("node2", "node1")?, false);
        Ok(())
    }

    #[derive(Debug, Copy, Clone)]
    struct ComponentA(u32);

    #[derive(Debug, Copy, Clone)]
    struct ComponentB(u32);

    #[test]
    fn test_add_node_duplicate_component() {
        let mut builder = DescriptionBuilder::new();

        // Adding a node with ComponentA
        builder
            .add_node("node1".to_string(), (ComponentA(10),))
            .unwrap();

        // Attempting to add ComponentA again to the same node should result in an error
        let result = builder.add_node("node1".to_string(), (ComponentA(20),));
        assert!(result.is_err());

        // However, adding a different component type (ComponentB) should be fine
        let result = builder.add_node("node1".to_string(), (ComponentB(30),));
        assert!(result.is_ok());
    }
}
