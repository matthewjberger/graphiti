use legion::{Entity, World};
use petgraph::graph::DiGraph;
use snafu::{OptionExt, Snafu};
use std::collections::HashMap;

/// Enumerates potential errors during construction.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Node '{}' not found", name))]
    NodeNotFound { name: String },

    #[snafu(display("Invalid parameters"))]
    InvalidParameters,

    #[snafu(display("Invalid edge name"))]
    InvalidEdgeName,
}

/// Represents the finalized structure of the machine.
#[derive(Debug)]
pub struct Description {
    pub data: World,
    pub graphs: HashMap<String, DiGraph<Entity, String>>,
}

/// A utility for constructing the `Description`.
pub struct DescriptionBuilder {
    world: World,
    node_indices: HashMap<String, Entity>,
    graphs: GraphContainer,
}

impl DescriptionBuilder {
    /// Create a new machine builder.
    pub fn new() -> Self {
        Self {
            world: World::default(),
            node_indices: HashMap::new(),
            graphs: GraphContainer::new(),
        }
    }

    /// Add a node (entity) to the world.
    pub fn add_node(&mut self, name: String, value: String) -> Result<&mut Self, Error> {
        if name.is_empty() || value.is_empty() {
            return Err(Error::InvalidParameters);
        }
        let entity = self.world.push((value,));
        self.node_indices.insert(name, entity);
        Ok(self)
    }

    /// Create connections (edges) between nodes for a given graph.
    pub fn add_edge(
        &mut self,
        edge_name: String,
        source: String,
        targets: Vec<String>,
    ) -> Result<&mut Self, Error> {
        if edge_name.is_empty() {
            return Err(Error::InvalidEdgeName);
        }
        self.graphs
            .add_edge(edge_name, source, &self.node_indices, targets)?;
        Ok(self)
    }

    /// Finalize and retrieve the machine description.
    pub fn build(self) -> Description {
        Description {
            data: self.world,
            graphs: self.graphs.graphs,
        }
    }
}

/// Container to manage directed graphs in the machine.
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
    ) -> Result<(), Error> {
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
            $($node_name:ident : $node_value:expr),* $(,)*
        },
        edges: {
            $($edge_name:literal : {
                $($source:ident : [$($target:ident),* $(,)*]),* $(,)*
            }),*
        }
    ) => {
        {
            let mut builder = graphiti::DescriptionBuilder::new();

            // Add nodes first
            $(builder.add_node(stringify!($node_name).to_string(), $node_value.to_string()).unwrap();)*

            // Now, add edges
            $(
                $(
                    builder.add_edge($edge_name.to_string(), stringify!($source).to_string(), vec![$(stringify!($target).to_string()),*]).unwrap();
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
    fn test_add_node() {
        let mut builder = DescriptionBuilder::new();
        builder
            .add_node("test_node".to_string(), "test_value".to_string())
            .unwrap();
        assert!(builder.node_indices.contains_key("test_node"));
    }

    #[test]
    fn test_add_edge() {
        let mut builder = DescriptionBuilder::new();
        builder
            .add_node("source".to_string(), "value1".to_string())
            .unwrap();
        builder
            .add_node("target".to_string(), "value2".to_string())
            .unwrap();
        let result = builder.add_edge(
            "edge1".to_string(),
            "source".to_string(),
            vec!["target".to_string()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_edge_missing_node() {
        let mut builder = DescriptionBuilder::new();
        builder
            .add_node("source".to_string(), "value1".to_string())
            .unwrap();
        let result = builder.add_edge(
            "edge1".to_string(),
            "source".to_string(),
            vec!["missing_target".to_string()],
        );
        assert!(result.is_err());
    }
}
