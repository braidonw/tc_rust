use std::collections::{HashMap, HashSet};

use super::{Graph, Node};

#[derive(Debug)]
pub struct Bridge {
    pub from: Node,
    pub to: Node,
}

#[derive(Debug)]
pub struct BridgeFinderState {
    pub visited: HashSet<Node>,
    pub low: HashMap<Node, usize>,
    pub discovery: HashMap<Node, usize>,
    pub parent: HashMap<Node, Node>,
    pub bridges: Vec<Bridge>,
}

impl BridgeFinderState {
    pub fn new(nodes: Vec<&Node>) -> BridgeFinderState {
        let num_nodes = nodes.len();
        BridgeFinderState {
            visited: HashSet::with_capacity(num_nodes),
            low: HashMap::with_capacity(num_nodes),
            discovery: HashMap::with_capacity(num_nodes),
            parent: HashMap::with_capacity(num_nodes),
            // We know roughly how big this will be
            bridges: Vec::with_capacity(5000),
        }
    }
}

pub trait BridgeFinder {
    fn find_bridges(&self) -> Vec<Bridge>;
    fn is_valid_bridge(&self, bridge: &Bridge) -> bool;

    fn dfs(
        &self,
        graph: &Graph,
        current: Node,
        parent: Node,
        time: usize,
        state: &mut BridgeFinderState,
    ) {
        state.visited.insert(current.clone());
        state.discovery.insert(current.clone(), time);
        state.low.insert(current.clone(), time);

        for next in graph.adjacent_nodes(&current) {
            if next.clone() == parent {
                continue;
            }

            if !state.visited.contains(&next) {
                self.dfs(graph, next.clone(), current.clone(), time + 1, state);
                state
                    .low
                    .insert(current.clone(), state.low[&current].min(state.low[&next]));

                // TODO: Check if the bridges are valid
                if state.low[&next] > state.discovery[&current] {
                    let bridge = Bridge {
                        from: current.clone(),
                        to: next.clone(),
                    };

                    if self.is_valid_bridge(&bridge) {
                        state.bridges.push(bridge);
                    };
                }
            } else {
                state.low.insert(
                    current.clone(),
                    state.low[&current].min(state.discovery[&next]),
                );
            }
        }
    }
}
