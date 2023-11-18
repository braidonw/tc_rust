use std::collections::HashMap;

use crate::{
    bridge_finder::{Bridge, BridgeFinder, BridgeFinderState},
    node::Node,
    union_find::UnionFind,
};

#[derive(Debug)]
pub struct Graph {
    pub edges: HashMap<Node, HashMap<Node, usize>>,
    pub roots: HashMap<Node, Node>,
    pub sizes: HashMap<Node, usize>,
    pub components: usize,
}

impl Graph {
    pub fn new(size: usize) -> Self {
        Graph {
            edges: HashMap::with_capacity(size),
            roots: HashMap::with_capacity(size),
            sizes: HashMap::new(),
            components: 0,
        }
    }

    pub fn nodes(&self) -> Vec<&Node> {
        self.edges.keys().collect::<Vec<&Node>>()
    }

    pub fn add_connected_component(&mut self, nodes: &[Node]) {
        for node in nodes {
            for other_node in nodes {
                if node == other_node {
                    continue;
                }
                self.add_edge(node, other_node);
            }
            self.roots.insert(node.clone(), node.clone());
        }
    }

    fn add_edge(&mut self, from: &Node, to: &Node) {
        let edges = self.edges.entry(from.clone()).or_insert(HashMap::new());
        let weight = edges.entry(to.clone()).or_insert(0);
        *weight += 1;
    }

    pub fn remove_edge(&mut self, from: &Node, to: &Node) {
        let edges = self.edges.get_mut(from).unwrap();
        edges.remove(to);
    }

    pub fn adjacent_nodes(&self, node: &Node) -> Vec<Node> {
        self.edges
            .get(node)
            .map(|edges| edges.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl BridgeFinder for Graph {
    fn find_bridges(&self) -> Vec<Bridge> {
        let mut state = BridgeFinderState::new(self.nodes());
        for (time, node) in self.nodes().into_iter().enumerate() {
            self.dfs(self, node.clone(), node.clone(), time, &mut state);
        }

        state.bridges
    }

    fn is_valid_bridge(&self, bridge: &Bridge) -> bool {
        let edge_weight = self
            .edges
            .get(&bridge.from)
            .and_then(|edges| edges.get(&bridge.to))
            // We know the edge exists, so unwrap is fine
            .expect("Edge weight not found");

        // Want to make sure that the weight of the edge between the two nodes is 1
        let edge_weight_is_one = *edge_weight == 1;

        let from_has_many_edges = self.edges.get(&bridge.from).unwrap().len() > 1;
        let to_has_many_edges = self.edges.get(&bridge.to).unwrap().len() > 1;

        edge_weight_is_one && from_has_many_edges && to_has_many_edges
    }
}

impl UnionFind<Node> for Graph {
    fn find(&mut self, node: &Node) -> Node {
        let mut root = node.clone();

        loop {
            if root == self.roots[&root] {
                break;
            }

            root = self.roots[&root].clone();
        }

        // Path compression
        let mut original = node.clone();
        loop {
            if original == root {
                break;
            }

            let next = self.roots[node].clone();
            self.roots.insert(node.clone(), root.clone());
            original = next;
        }

        root
    }

    fn union(&mut self, node1: &Node, node2: &Node) {
        let root1 = self.find(node1);
        let root2 = self.find(node2);

        if root1 == root2 {
            return;
        }

        let root1_size = *self.sizes.entry(root1.clone()).or_insert(1);
        let root2_size = *self.sizes.entry(root2.clone()).or_insert(1);

        let new_size = root1_size + root2_size;

        if root1_size <= root2_size {
            self.roots.insert(root1.clone(), root2.clone());
            self.sizes.insert(root2, new_size);
        } else {
            self.roots.insert(root2.clone(), root1.clone());
            self.sizes.insert(root1, new_size);
        }

        self.components -= 1;
    }
}
