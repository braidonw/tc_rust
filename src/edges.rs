use crate::node::Node;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Edges(HashMap<Node, u32>);

impl Edges {
    pub fn new() -> Self {
        Edges(HashMap::new())
    }

    pub fn add_or_increment(&mut self, node: Node) {
        let count = self.0.entry(node).or_insert(0);
        *count += 1;
    }

    pub fn get_weight(&self, node: &Node) -> Option<&u32> {
        self.0.get(node)
    }

    pub fn remove_edge(&mut self, node: &Node) {
        self.0.remove(node);
    }

    pub fn nodes(&self) -> Vec<Node> {
        self.0.keys().cloned().collect()
    }
}
