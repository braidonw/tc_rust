use std::collections::HashMap;

use crate::{graph::Graph, node::Node};
#[allow(dead_code)]
#[derive(Debug)]
struct UnionFind<'a> {
    graph: &'a Graph,
    sizes: HashMap<&'a Node, usize>,
    parents: HashMap<&'a Node, &'a Node>,
    components: usize,
}

#[allow(dead_code)]
impl<'a> UnionFind<'a> {
    fn new(graph: &'a Graph) -> Self {
        let num_nodes = graph.edges.len();
        let mut uf = UnionFind {
            graph,
            sizes: HashMap::with_capacity(num_nodes),
            parents: HashMap::with_capacity(num_nodes),
            // Just init this value to the number of nodes, rather than incrementing it by one each
            // loop below
            components: num_nodes,
        };

        for node in graph.nodes() {
            uf.sizes.insert(node, 1);
            uf.parents.insert(node, node);
        }

        uf
    }

    fn parent(&self, node: &Node) -> Option<&Node> {
        self.parents.get(node).copied()
    }

    fn set_parent(&mut self, node: &'a Node, parent: &'a Node) {
        self.parents.insert(node, parent);
    }

    fn sizeof(&self, node: &Node) -> Option<usize> {
        self.sizes.get(node).copied()
    }

    fn root(&mut self, node: &'a mut Node) -> &Node {
        let mut root = node;

        while self.parent(root) != Some(root) {
            root = &mut *self.parent(root).expect("No parent");
        }

        let mut init_node = &mut node.clone();
        // Path compression
        while init_node != root {
            let mut next = self.parent(node).expect("No parent").clone();
            self.set_parent(node, root);
            init_node = &mut next;
        }

        root
    }
}
