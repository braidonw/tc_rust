mod edges;
mod graph;
mod graph2;
mod input;
mod node;
use graph::Graph;
mod bridges;
mod union_find;

fn main() -> anyhow::Result<()> {
    let mut graph = Graph::new();
    let records = input::parse("./data.csv")?;
    let nodes = records
        .iter()
        .map(|record| {
            record
                .node_values()
                .into_iter()
                .map(|(kind, value)| node::Node::new(kind, value).expect("Invalid node"))
                .collect::<Vec<node::Node>>()
        })
        .collect::<Vec<Vec<node::Node>>>();

    for group in nodes.iter() {
        graph.add_connected_graph(group);
    }

    dbg!(&graph.nodes().len());

    let bridges = bridges::find(&graph);

    dbg!(bridges.len());

    Ok(())
}
