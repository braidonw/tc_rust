mod bridge_finder;
mod graph;
mod input;
mod node;
mod union_find;

use crate::{graph::Graph, node::Node};
use bridge_finder::BridgeFinder;
use union_find::UnionFind;

#[allow(unused_variables)]
fn main() -> anyhow::Result<()> {
    let mut graph = Graph::new(200000);
    let records = input::parse("./data.csv")?;
    let nodes = records
        .iter()
        .map(|record| {
            record
                .node_values()
                .into_iter()
                .map(|(kind, value)| Node::new(kind, value).expect("Invalid node"))
                .collect::<Vec<Node>>()
        })
        .collect::<Vec<Vec<Node>>>();

    for group in nodes.iter() {
        graph.add_connected_component(group);
    }

    // Set the number of components
    graph.components = graph.nodes().len();

    // Find and remove valid bridges from the graph
    let bridges = graph.find_bridges();
    for bridge in bridges {
        graph.remove_edge(&bridge.from, &bridge.to);
        graph.remove_edge(&bridge.to, &bridge.from);
    }

    // Run Union Find on the graph, showing the starting and ending components
    dbg!(&graph.components);
    let nodes = graph.edges.keys().cloned().collect::<Vec<Node>>();
    nodes.iter().for_each(|node| {
        graph.adjacent_nodes(node).iter().for_each(|adjacent_node| {
            graph.union(node, adjacent_node);
        });
    });
    dbg!(&graph.components);

    Ok(())
}
