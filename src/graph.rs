use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    error::Error,
    fmt::Display,
    hash::Hash,
};

#[derive(Debug)]
pub enum EntityGraphError {
    EntityAlreadyExists,
    EntityNotFound,
    EdgeError,
    SerializationError(String),
    DeserializationError(String),
}

impl Display for EntityGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EntityGraphError::EntityAlreadyExists => {
                write!(f, "Entity with this ID already exists")
            }
            EntityGraphError::EntityNotFound => write!(f, "Entity with this ID does not exist"),
            EntityGraphError::EdgeError => write!(f, "One of the entity IDs does not exist"),
            EntityGraphError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            EntityGraphError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
        }
    }
}

impl Error for EntityGraphError {}

pub trait EntityId: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> {}
impl<T> EntityId for T where T: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> {}

pub trait MapKey: Eq + Hash + Clone {}
impl<T> MapKey for T where T: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> {}

type Entities<ID, K> = HashMap<ID, HashMap<K, Value>>;
type Relationships<ID, R> = HashMap<R, AdjacencyList<ID>>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EntityGraph<ID: Eq + Hash + Clone, K: Eq + Hash + Clone, R: Eq + Hash + Clone> {
    entities: Entities<ID, K>,
    relationships: Relationships<ID, R>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AdjacencyList<ID: Eq + Hash + Clone> {
    edges: HashMap<ID, Vec<ID>>,
}

impl<ID, K, R> EntityGraph<ID, K, R>
where
    ID: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
    K: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Display,
    R: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Display,
{
    pub fn new() -> Self {
        EntityGraph {
            entities: HashMap::new(),
            relationships: HashMap::new(),
        }
    }

    pub fn add_entity(
        &mut self,
        id: ID,
        components: HashMap<K, Value>,
    ) -> Result<(), EntityGraphError> {
        if self.entities.contains_key(&id) {
            return Err(EntityGraphError::EntityAlreadyExists);
        }
        self.entities.insert(id, components);
        Ok(())
    }

    pub fn remove_entity(&mut self, id: &ID) {
        // Remove the entity from the entities HashMap
        self.entities.remove(id);

        // Remove the entity from all relationships in the relationships HashMap
        for (_relationship_key, adjacency_list) in &mut self.relationships {
            adjacency_list.edges.remove(id);
            // Additionally, remove the entity from the list of neighbors in all adjacency lists
            for neighbors in adjacency_list.edges.values_mut() {
                neighbors.retain(|neighbor_id| neighbor_id != id);
            }
        }
    }

    pub fn add_edge(
        &mut self,
        relationship_key: R,
        from: ID,
        to: ID,
    ) -> Result<(), EntityGraphError> {
        if !self.entities.contains_key(&from) || !self.entities.contains_key(&to) {
            return Err(EntityGraphError::EdgeError);
        }

        // Get or create the adjacency list for the given relationship_key
        let adjacency_list = self
            .relationships
            .entry(relationship_key)
            .or_insert_with(|| AdjacencyList {
                edges: HashMap::new(),
            });

        // Add the edge to the adjacency list
        adjacency_list
            .edges
            .entry(from)
            .or_insert_with(Vec::new)
            .push(to);

        Ok(())
    }

    pub fn serialize(&self) -> Result<String, Box<dyn Error>> {
        serde_json::to_string(&self).map_err(Into::into)
    }

    pub fn deserialize_with_registry(
        data: &str,
        registry: &TypeRegistry,
    ) -> Result<Self, EntityGraphError> {
        let mut graph: Self = serde_json::from_str(data).map_err(|e| {
            EntityGraphError::DeserializationError(format!("Failed to deserialize graph: {}", e))
        })?;

        // Deserialize components
        for (_id, component_map) in graph.entities.iter_mut() {
            for (type_name, value) in component_map.iter_mut() {
                match registry.deserialize_value(&type_name.to_string(), value) {
                    Ok(new_value) => *value = new_value,
                    Err(e) => {
                        return Err(EntityGraphError::DeserializationError(format!(
                            "Failed to deserialize component: {}",
                            e
                        )))
                    }
                }
            }
        }

        Ok(graph)
    }
    pub fn traverse_dfs(&self, start: ID) -> Option<Vec<ID>> {
        let mut visited = HashMap::new();
        let mut stack = vec![start];
        let mut result = Vec::new();

        while let Some(current) = stack.pop() {
            if !visited.contains_key(&current) {
                visited.insert(current.clone(), true);
                result.push(current.clone());

                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors {
                        if !visited.contains_key(neighbor) {
                            stack.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn traverse_bfs(&self, start: ID) -> Option<Vec<ID>> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(start.clone());
        visited.insert(start.clone(), true);

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            if let Some(neighbors) = self.get_neighbors(&current) {
                for neighbor in neighbors {
                    if !visited.contains_key(neighbor) {
                        visited.insert(neighbor.clone(), true);
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn get_neighbors(&self, entity_id: &ID) -> Option<&Vec<ID>> {
        for adjacency_list in self.relationships.values() {
            if let Some(neighbors) = adjacency_list.edges.get(entity_id) {
                return Some(neighbors);
            }
        }
        None
    }

    pub fn get_component(&self, entity_id: &ID, component_key: &K) -> Option<&Value> {
        self.entities
            .get(entity_id)
            .and_then(|components| components.get(component_key))
    }
}

#[cfg(feature = "petgraph")]
fn entity_graph_to_petgraph_directed_graphs<
    ID: Clone + Eq + Hash + Serialize + for<'de> Deserialize<'de>,
>(
    entity_graph: &EntityGraph<ID>,
) -> Vec<petgraph::graph::DiGraph<ID, ()>> {
    let mut graphs = Vec::new();

    for adjacency_list in &entity_graph.relationships {
        let mut graph = petgraph::graph::DiGraph::new();
        let mut node_indices = HashMap::new();

        for (node_id, neighbors) in &adjacency_list.edges {
            let source_index = *node_indices
                .entry(node_id.clone())
                .or_insert_with(|| graph.add_node(node_id.clone()));

            for neighbor in neighbors {
                let target_index = *node_indices
                    .entry(neighbor.clone())
                    .or_insert_with(|| graph.add_node(neighbor.clone()));
                graph.add_edge(source_index, target_index, ());
            }
        }

        graphs.push(graph);
    }

    graphs
}

pub struct TypeRegistry {
    deserialize_fn_map: HashMap<String, Box<dyn Fn(&Value) -> Result<Box<dyn Any + Send>, String>>>,
    serialize_map: HashMap<String, Box<dyn Fn(&(dyn Any + Send)) -> Option<Value>>>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            deserialize_fn_map: HashMap::new(),
            serialize_map: HashMap::new(),
        }
    }

    // Register a type with its serialization function

    pub fn register<T: 'static + Send + Serialize + DeserializeOwned>(&mut self, type_name: &str) {
        self.serialize_map.insert(
            type_name.to_string(),
            Box::new(move |any: &(dyn Any + Send)| {
                any.downcast_ref::<T>()
                    .and_then(|typed_ref| serde_json::to_value(typed_ref).ok())
            }),
        );

        self.deserialize_fn_map.insert(
            type_name.to_string(),
            Box::new(move |value: &Value| {
                serde_json::from_value::<T>(value.clone())
                    .map(|value| Box::new(value) as Box<dyn Any + Send>)
                    .map_err(|e| e.to_string())
            }),
        );
    }

    pub fn deserialize_value(&self, type_name: &str, value: &Value) -> Result<Value, String> {
        // Deserialize using the appropriate function from the map
        if let Some(deserialize_fn) = self.deserialize_fn_map.get(type_name) {
            let deserialized_value = deserialize_fn(value);

            // Attempt to re-serialize the deserialized value
            if let Some(serialize_fn) = self.serialize_map.get(type_name) {
                serialize_fn(&*deserialized_value?)
                    .ok_or_else(|| format!("Failed to re-serialize for: {}", type_name))
            } else {
                Err(format!(
                    "No serialization function found for type: {}",
                    type_name
                ))
            }
        } else {
            Err(format!(
                "No deserialization function found for type: {}",
                type_name
            ))
        }
    }
}

#[macro_export]
macro_rules! register_types {
    ($registry:expr, $(($t:ty, $s:expr)),* ) => {
        $(
            $registry.register::<$t>($s);
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    type TestGraph = EntityGraph<String, String, String>;

    #[test]
    fn test_add_remove_entity() {
        let mut graph = TestGraph::new();
        assert!(graph
            .add_entity(
                "entity1".to_string(),
                vec![
                    ("component_name1".to_string(), Value::from("component1")),
                    ("component_name2".to_string(), Value::from("component2"))
                ]
                .into_iter()
                .collect()
            )
            .is_ok());
        assert!(graph
            .add_entity(
                "entity1".to_string(),
                vec![("component_name3".to_string(), Value::from("component3"))]
                    .into_iter()
                    .collect()
            )
            .is_err());

        graph.remove_entity(&"entity1".to_string());
        assert_eq!(graph.entities.contains_key(&"entity1".to_string()), false);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = TestGraph::new();
        graph
            .add_entity(
                "entity1".to_string(),
                vec![("component_name1".to_string(), Value::from("component1"))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
        graph
            .add_entity(
                "entity2".to_string(),
                vec![("component_name2".to_string(), Value::from("component2"))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();

        assert!(graph
            .add_edge(
                "relationship".to_string(),
                "entity1".to_string(),
                "entity2".to_string()
            )
            .is_ok());
        assert!(graph
            .add_edge(
                "relationship".to_string(),
                "entity1".to_string(),
                "entity3".to_string()
            )
            .is_err());
    }

    #[cfg(feature = "petgraph")]
    #[test]
    fn test_entity_graph_to_petgraph_conversion() {
        let mut graph = EntityGraph::<String, String>::new();
        graph
            .add_entity(
                "entity1".to_string(),
                vec![("component_name1".to_string(), Value::from("component1"))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
        graph
            .add_entity(
                "entity2".to_string(),
                vec![("component_name2".to_string(), Value::from("component2"))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
        graph
            .add_edge("entity1".to_string(), "entity2".to_string())
            .unwrap();

        let petgraphs = entity_graph_to_petgraph_directed_graphs(&graph);

        assert_eq!(petgraphs.len(), 1);
        let petgraph = &petgraphs[0];
        assert_eq!(petgraph.node_count(), 2);
        assert_eq!(petgraph.edge_count(), 1);
    }

    // Mock ECS setup
    mod mock_ecs {
        use serde_json::Value;
        use std::collections::HashMap;

        #[derive(Default)]
        pub struct World {
            pub entities: Vec<Entity>,
        }

        #[derive(Default)]
        pub struct Entity {
            pub components: HashMap<String, Value>,
        }

        impl World {
            pub fn new() -> Self {
                World {
                    entities: Vec::new(),
                }
            }

            pub fn create_entity(&mut self) -> &mut Entity {
                self.entities.push(Entity::default());
                self.entities.last_mut().unwrap()
            }
        }

        impl Entity {
            pub fn add_component(&mut self, key: &str, component: Value) {
                self.components.insert(key.to_string(), component);
            }
        }
    }

    #[test]
    fn test_populate_mock_ecs_with_entity_graph() {
        let mut graph = TestGraph::new();
        graph
            .add_entity(
                "entity1".to_string(),
                vec![
                    ("position".to_string(), Value::from("x:10, y:20")),
                    ("velocity".to_string(), Value::from("dx:5, dy:-5")),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let mut world = mock_ecs::World::new();

        for (_id, components) in &graph.entities {
            let entity = world.create_entity();
            for (component_name, component_data) in components {
                entity.add_component(component_name, component_data.clone());
            }
        }

        assert_eq!(world.entities.len(), 1);
        let mock_entity = &world.entities[0];
        assert_eq!(
            mock_entity.components.get("position").unwrap(),
            &Value::from("x:10, y:20")
        );
        assert_eq!(
            mock_entity.components.get("velocity").unwrap(),
            &Value::from("dx:5, dy:-5")
        );
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    pub struct Component5 {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_serialization_and_deserialization() {
        let mut graph = TestGraph::new();
        graph
            .add_entity(
                "entity1".to_string(),
                vec![
                    ("component_name1".to_string(), Value::from("component1")),
                    ("component_name2".to_string(), Value::from(1234)),
                    ("component_name3".to_string(), Value::from(true)),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        graph
            .add_entity(
                "entity2".to_string(),
                vec![("component_name4".to_string(), Value::from(5.67))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
        graph
            .add_edge(
                "relationship".to_string(),
                "entity1".to_string(),
                "entity2".to_string(),
            )
            .unwrap();
        // Create an instance of Component5 and serialize it as a component for an entity
        let comp5 = Component5 {
            field1: "some_data".to_string(),
            field2: 42,
        };
        graph
            .add_entity(
                "entity3".to_string(),
                vec![(
                    "component_name5".to_string(),
                    serde_json::to_value(&comp5).unwrap(),
                )]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let serialized = graph.serialize().unwrap();

        // Here we set up the type registry for deserialization
        let mut registry = TypeRegistry::new();
        register_types!(
            registry,
            (String, "component_name1"),
            (i32, "component_name2"),
            (bool, "component_name3"),
            (f64, "component_name4"),
            (Component5, "component_name5")
        );

        let deserialized = TestGraph::deserialize_with_registry(&serialized, &registry).unwrap();

        assert_eq!(graph, deserialized);
    }

    #[test]
    fn test_dfs_traversal() {
        let mut graph = TestGraph::new();

        // Adding entities
        graph.add_entity("A".to_string(), HashMap::new()).unwrap();
        graph.add_entity("B".to_string(), HashMap::new()).unwrap();
        graph.add_entity("C".to_string(), HashMap::new()).unwrap();
        graph.add_entity("D".to_string(), HashMap::new()).unwrap();

        // Adding edges
        graph
            .add_edge("relationship".to_string(), "A".to_string(), "B".to_string())
            .unwrap();
        graph
            .add_edge("relationship".to_string(), "A".to_string(), "C".to_string())
            .unwrap();
        graph
            .add_edge("relationship".to_string(), "B".to_string(), "D".to_string())
            .unwrap();

        let traversal_result = graph.traverse_dfs("A".to_string()).unwrap();
        let expected_traversal = vec![
            "A".to_string(),
            "C".to_string(),
            "B".to_string(),
            "D".to_string(),
        ];

        assert_eq!(traversal_result, expected_traversal);
    }

    #[test]
    fn test_bfs_traversal() {
        let mut graph = TestGraph::new();

        // Adding entities
        graph.add_entity("A".to_string(), HashMap::new()).unwrap();
        graph.add_entity("B".to_string(), HashMap::new()).unwrap();
        graph.add_entity("C".to_string(), HashMap::new()).unwrap();
        graph.add_entity("D".to_string(), HashMap::new()).unwrap();

        // Adding edges
        graph
            .add_edge("relationship".to_string(), "A".to_string(), "B".to_string())
            .unwrap();
        graph
            .add_edge("relationship".to_string(), "A".to_string(), "C".to_string())
            .unwrap();
        graph
            .add_edge("relationship".to_string(), "B".to_string(), "D".to_string())
            .unwrap();

        let traversal_result = graph.traverse_bfs("A".to_string()).unwrap();
        let expected_traversal = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];

        assert_eq!(traversal_result, expected_traversal);
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
    enum ComponentKey {
        Position,
        Velocity,
    }

    impl std::fmt::Display for ComponentKey {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[test]
    fn test_enum_key() {
        let mut graph = EntityGraph::<String, ComponentKey, String>::new();
        graph
            .add_entity(
                "entity1".to_string(),
                vec![
                    (
                        ComponentKey::Position,
                        serde_json::Value::from("x:10, y:20"),
                    ),
                    (
                        ComponentKey::Velocity,
                        serde_json::Value::from("dx:5, dy:-5"),
                    ),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let position = graph.get_component(&"entity1".to_string(), &ComponentKey::Position);
        assert_eq!(position, Some(&serde_json::Value::from("x:10, y:20")));
    }

    #[test]
    fn test_dfs_print_components() {
        // Create an entity graph and add entities with components
        let mut graph = TestGraph::new();
        graph
            .add_entity(
                "A".to_string(),
                vec![
                    ("type1".to_string(), serde_json::Value::from("data1")),
                    ("type2".to_string(), serde_json::Value::from(123)),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        graph
            .add_entity(
                "B".to_string(),
                vec![("type1".to_string(), serde_json::Value::from("data2"))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
        graph.add_entity("C".to_string(), HashMap::new()).unwrap();

        // Add edges for traversal
        graph
            .add_edge("relationship".to_string(), "A".to_string(), "B".to_string())
            .unwrap();
        graph
            .add_edge("relationship".to_string(), "A".to_string(), "C".to_string())
            .unwrap();

        // Set up the type registry
        let mut registry = TypeRegistry::new();
        registry.register::<String>("type1");
        registry.register::<i32>("type2");

        // Perform DFS traversal and print components
        let traversal_result = graph.traverse_dfs("A".to_string()).unwrap();

        // Expected traversal order and component count
        let expected_order = vec!["A", "C", "B"];
        let expected_components = vec![2, 0, 1];

        // Check traversal order
        assert_eq!(traversal_result, expected_order);

        for (index, entity_id) in traversal_result.iter().enumerate() {
            if let Some(components) = graph.entities.get(entity_id) {
                // Assert the number of components
                assert_eq!(components.len(), expected_components[index]);

                // Bonus: Check component types
                for (type_name, value) in components {
                    assert!(
                        registry.deserialize_value(type_name, value).is_ok(),
                        "Component of type {} is NOT of expected type.",
                        type_name
                    );
                }
            }
        }
    }
}
