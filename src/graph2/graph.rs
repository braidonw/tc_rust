use crate::edges::Edges;
use crate::graph2::Node;

#[derive(Debug)]
pub struct Graph<'a> {
    pub nodes: Vec<Node<'a>>,
    pub root_nodes: Vec<&'a Node<'a>>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            root_nodes: Vec::new(),
        }
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
