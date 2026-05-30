use crate::graph::Graph;
use crate::node::NodeId;

/// Sentinel "no parent" marker for DFS roots. Node ids are dense indices well
/// below `u32::MAX`, so this never collides with a real id.
const NO_PARENT: NodeId = NodeId::MAX;
const UNVISITED: u32 = u32::MAX;

/// One entry of the explicit DFS stack, replacing the recursive call frame.
struct Frame {
    node: NodeId,
    parent: NodeId,
    /// Weight of the tree edge from `parent` to `node` (unused for roots).
    parent_weight: u32,
    /// Index of the next neighbour of `node` to visit.
    next: usize,
}

/// Finds the *valid* bridges of `graph` using an iterative Tarjan low-link DFS.
///
/// A bridge is an edge whose removal disconnects the graph. We keep only the
/// bridges worth cutting for entity resolution: those asserted by a single
/// record (`weight == 1`) whose **both** endpoints have other connections
/// (`degree > 1`), so cutting them separates two genuine clusters rather than
/// orphaning a leaf identifier.
///
/// The DFS is iterative (an explicit `Vec` stack) so it cannot overflow the
/// call stack on deep components, and it uses a single global discovery
/// counter — the canonical formulation.
pub fn find_valid_bridges(graph: &Graph) -> Vec<(NodeId, NodeId)> {
    let n = graph.num_nodes();
    let mut disc = vec![UNVISITED; n];
    let mut low = vec![UNVISITED; n];
    let mut timer: u32 = 0;
    let mut bridges = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();

    for start in 0..n as NodeId {
        if disc[start as usize] != UNVISITED {
            continue;
        }

        disc[start as usize] = timer;
        low[start as usize] = timer;
        timer += 1;
        stack.push(Frame {
            node: start,
            parent: NO_PARENT,
            parent_weight: 0,
            next: 0,
        });

        while let Some(frame) = stack.last_mut() {
            let u = frame.node;
            let neighbors = graph.neighbors(u);

            if frame.next < neighbors.len() {
                let (v, weight) = neighbors[frame.next];
                frame.next += 1;

                // The single edge back to the parent is the tree edge, not a
                // back edge — skip it once.
                if v == frame.parent {
                    continue;
                }

                if disc[v as usize] == UNVISITED {
                    // Tree edge: descend into v.
                    disc[v as usize] = timer;
                    low[v as usize] = timer;
                    timer += 1;
                    stack.push(Frame {
                        node: v,
                        parent: u,
                        parent_weight: weight,
                        next: 0,
                    });
                } else {
                    // Back edge: v is an ancestor; pull its discovery time into low[u].
                    low[u as usize] = low[u as usize].min(disc[v as usize]);
                }
            } else {
                // Finished u: pop it and fold its low-link into its parent,
                // testing the parent→u tree edge for being a bridge.
                let done = stack.pop().unwrap();
                if let Some(parent_frame) = stack.last() {
                    let p = parent_frame.node;
                    low[p as usize] = low[p as usize].min(low[u as usize]);

                    let is_bridge = low[u as usize] > disc[p as usize];
                    let is_valid =
                        done.parent_weight == 1 && graph.degree(p) > 1 && graph.degree(u) > 1;
                    if is_bridge && is_valid {
                        bridges.push((p.min(u), p.max(u)));
                    }
                }
            }
        }
    }

    bridges
}

#[cfg(test)]
mod test {
    use super::find_valid_bridges;
    use crate::graph::GraphBuilder;

    fn sorted(mut v: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
        v.sort_unstable();
        v
    }

    #[test]
    fn triangle_has_no_bridges() {
        let mut b = GraphBuilder::new(3);
        b.add_clique(&[0, 1, 2]);
        assert!(find_valid_bridges(&b.build()).is_empty());
    }

    #[test]
    fn clique_of_four_has_no_bridges() {
        let mut b = GraphBuilder::new(4);
        b.add_clique(&[0, 1, 2, 3]);
        assert!(find_valid_bridges(&b.build()).is_empty());
    }

    #[test]
    fn leaf_edges_are_structural_bridges_but_not_valid() {
        // Path 0-1-2: every bridge has a degree-1 endpoint, so none are valid.
        let mut b = GraphBuilder::new(3);
        b.add_clique(&[0, 1]);
        b.add_clique(&[1, 2]);
        assert!(find_valid_bridges(&b.build()).is_empty());
    }

    #[test]
    fn interior_bridge_with_two_real_endpoints_is_valid() {
        // Path 0-1-2-3: only the middle edge (1,2) has degree>1 on both ends.
        let mut b = GraphBuilder::new(4);
        b.add_clique(&[0, 1]);
        b.add_clique(&[1, 2]);
        b.add_clique(&[2, 3]);
        assert_eq!(find_valid_bridges(&b.build()), vec![(1, 2)]);
    }

    #[test]
    fn single_weak_link_between_clusters_is_a_valid_bridge() {
        // Two triangles joined by one weight-1 edge (2,3).
        let mut b = GraphBuilder::new(6);
        b.add_clique(&[0, 1, 2]);
        b.add_clique(&[3, 4, 5]);
        b.add_clique(&[2, 3]);
        assert_eq!(find_valid_bridges(&b.build()), vec![(2, 3)]);
    }

    #[test]
    fn double_asserted_link_is_not_a_valid_bridge() {
        // Same join, but two records assert (2,3): weight 2, so not cut.
        let mut b = GraphBuilder::new(6);
        b.add_clique(&[0, 1, 2]);
        b.add_clique(&[3, 4, 5]);
        b.add_clique(&[2, 3]);
        b.add_clique(&[2, 3]);
        assert!(find_valid_bridges(&b.build()).is_empty());
    }

    #[test]
    fn isolated_nodes_have_no_bridges() {
        let mut b = GraphBuilder::new(3);
        b.add_clique(&[0]);
        assert!(find_valid_bridges(&b.build()).is_empty());
    }

    #[test]
    fn finds_bridges_across_multiple_components() {
        // Two independent "two-triangles-joined" components in one graph.
        let mut b = GraphBuilder::new(12);
        b.add_clique(&[0, 1, 2]);
        b.add_clique(&[3, 4, 5]);
        b.add_clique(&[2, 3]);
        b.add_clique(&[6, 7, 8]);
        b.add_clique(&[9, 10, 11]);
        b.add_clique(&[8, 9]);
        assert_eq!(sorted(find_valid_bridges(&b.build())), vec![(2, 3), (8, 9)]);
    }
}
