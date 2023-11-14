use std::collections::{HashMap, HashSet};

use crate::{graph::Graph, node::Node};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bridge<'a> {
    from: &'a Node,
    to: &'a Node,
}

#[derive(Debug)]
struct BridgeFinder<'a> {
    graph: &'a Graph,
    bridges: Vec<Bridge<'a>>,

    visited: HashSet<&'a Node>,
    low: HashMap<&'a Node, usize>,
    discovery: HashMap<&'a Node, usize>,
    parent: HashMap<&'a Node, &'a Node>,
}

impl<'a> BridgeFinder<'a> {
    pub fn new(graph: &'a Graph) -> BridgeFinder {
        let num_nodes = graph.edges.len();
        let mut bf = BridgeFinder {
            graph,
            bridges: Vec::with_capacity(num_nodes),
            visited: HashSet::with_capacity(num_nodes),
            low: HashMap::with_capacity(num_nodes),
            discovery: HashMap::with_capacity(num_nodes),
            parent: HashMap::with_capacity(num_nodes),
        };

        for node in graph.edges.keys() {
            bf.low.insert(node, 0);
            bf.discovery.insert(node, 0);
            bf.parent.insert(node, node);
        }

        bf
    }

    fn depth_first_search(&mut self, current: &'a Node, parent: &'a Node, time: usize) {
        self.visited.insert(current);
        self.discovery.insert(current, time);
        self.low.insert(current, time);

        for next in self.graph.adjacent_nodes(current) {
            if next == parent {
                continue;
            }

            if !self.visited.contains(next) {
                self.depth_first_search(next, current, time + 1);
                self.low
                    .insert(current, self.low[current].min(self.low[next]));

                if self.low[next] > self.discovery[current] {
                    self.bridges.push(Bridge {
                        from: current,
                        to: next,
                    });
                }
            } else if next != parent {
                self.low
                    .insert(current, self.low[current].min(self.discovery[next]));
            }
        }
    }
}

pub fn find(graph: &Graph) -> Vec<Bridge> {
    let mut bf = BridgeFinder::new(graph);

    for (time, node) in bf.graph.edges.keys().enumerate() {
        bf.depth_first_search(
            node,
            bf.parent
                .get(node)
                .expect("Node not found in parent hashmap"),
            time,
        );
    }

    bf.bridges.clone()
}
