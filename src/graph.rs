use crate::edges::Edges;
use crate::node::Node;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Graph {
    pub edges: HashMap<Node, Edges>,
    pub parents: HashMap<Node, Node>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            edges: HashMap::new(),
            parents: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: &Node, to: &Node) {
        let edges = self.edges.entry(from.clone()).or_insert(Edges::new());
        edges.add_or_increment(to.clone());
    }

    pub fn add_connected_graph(&mut self, nodes: &[Node]) {
        for node in nodes {
            for other_node in nodes {
                if node == other_node {
                    continue;
                }
                self.add_edge(node, other_node);
            }
        }
    }

    pub fn nodes(&self) -> Vec<&Node> {
        self.edges.keys().collect()
    }

    pub fn adjacent_nodes(&self, node: &Node) -> Vec<&Node> {
        self.edges
            .get(node)
            .map(|edges| edges.nodes())
            .unwrap_or_default()
    }
}
