macro_rules! describe {
    (nodes: { $( $node_name:ident: [ $( $comp_value:expr ),* ] ),* $(,)? }, edges: { $( $edge_name:tt: { $( $from_node:ident => [ $( $to_node:ident ),* ] ),* } ),* $(,)? }) => {
        {
            let mut builder = graphiti::DescriptionBuilder::new();

            // Add nodes
            $(
                builder.add_node(stringify!($node_name).to_string(), ($( $comp_value ),*))?;
            )*

            // Add edges
            $(
                $(
                    builder.add_edge(stringify!($edge_name), stringify!($from_node), vec![ $( stringify!($to_node), )* ])?;
                )*
            )*

            builder.build()
        }
    };
}

#[derive(Debug, Clone)]
pub struct NodeValue(String);

fn main() -> Result<(), graphiti::Error> {
    let description = describe! {
        nodes: {
            node1: [NodeValue("value1".to_string())],
            node2: [NodeValue("value2".to_string())],
            node3: []
        },
        edges: {
            "edge1": {
                node1 => [node2]
            },
            "edge2": {
                node2 => [node3]
            }
        }
    };

    // Use the description if needed
    println!("{:?}", description);

    Ok(())
}
