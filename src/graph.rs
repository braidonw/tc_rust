use crate::node::NodeId;
use rustc_hash::FxHashMap;

/// Undirected weighted graph of identifier nodes, stored as adjacency lists
/// indexed by `NodeId`. An edge's weight is the number of records that
/// co-asserted that pair of identifiers; it is built once and then read-only.
#[derive(Debug)]
pub struct Graph {
    /// `adjacency[id]` holds `(neighbor, weight)` for every edge incident to `id`.
    adjacency: Vec<Vec<(NodeId, u32)>>,
}

impl Graph {
    pub fn num_nodes(&self) -> usize {
        self.adjacency.len()
    }

    pub fn neighbors(&self, id: NodeId) -> &[(NodeId, u32)] {
        &self.adjacency[id as usize]
    }

    pub fn degree(&self, id: NodeId) -> usize {
        self.adjacency[id as usize].len()
    }

    /// Yields each undirected edge once as `(a, b, weight)` with `a < b`.
    pub fn edges(&self) -> impl Iterator<Item = (NodeId, NodeId, u32)> + '_ {
        self.adjacency
            .iter()
            .enumerate()
            .flat_map(|(a, neighbors)| {
                let a = a as NodeId;
                neighbors
                    .iter()
                    .filter(move |(b, _)| a < *b)
                    .map(move |&(b, w)| (a, b, w))
            })
    }
}

/// Accumulates edge weights across records, then freezes them into a `Graph`.
/// Weights are keyed by the canonical ordered pair `(min, max)` so the two
/// directions of an undirected edge share one counter.
#[derive(Debug)]
pub struct GraphBuilder {
    num_nodes: usize,
    edges: FxHashMap<(NodeId, NodeId), u32>,
}

impl GraphBuilder {
    pub fn new(num_nodes: usize) -> Self {
        GraphBuilder {
            num_nodes,
            edges: FxHashMap::default(),
        }
    }

    /// Adds a record's identifiers as a clique: every distinct pair gets its
    /// weight bumped by one. The ids within a record are already distinct (one
    /// per `NodeKind`), so the `a == b` guard is just defensive.
    pub fn add_clique(&mut self, ids: &[NodeId]) {
        for (i, &a) in ids.iter().enumerate() {
            for &b in &ids[i + 1..] {
                if a == b {
                    continue;
                }
                let key = if a < b { (a, b) } else { (b, a) };
                *self.edges.entry(key).or_insert(0) += 1;
            }
        }
    }

    pub fn build(self) -> Graph {
        let mut adjacency = vec![Vec::new(); self.num_nodes];
        for ((a, b), weight) in self.edges {
            adjacency[a as usize].push((b, weight));
            adjacency[b as usize].push((a, weight));
        }
        Graph { adjacency }
    }
}

#[cfg(test)]
mod test {
    use super::GraphBuilder;

    #[test]
    fn clique_creates_all_pairs() {
        let mut b = GraphBuilder::new(3);
        b.add_clique(&[0, 1, 2]);
        let g = b.build();
        assert_eq!(g.degree(0), 2);
        assert_eq!(g.degree(1), 2);
        assert_eq!(g.degree(2), 2);
        assert_eq!(g.edges().count(), 3);
    }

    #[test]
    fn shared_identifier_accumulates_weight() {
        // Two records both assert the pair (0, 1); a third asserts (1, 2) once.
        let mut b = GraphBuilder::new(3);
        b.add_clique(&[0, 1]);
        b.add_clique(&[0, 1]);
        b.add_clique(&[1, 2]);
        let g = b.build();
        let edge_01 = g.edges().find(|&(a, c, _)| a == 0 && c == 1).unwrap();
        assert_eq!(edge_01.2, 2);
        let edge_12 = g.edges().find(|&(a, c, _)| a == 1 && c == 2).unwrap();
        assert_eq!(edge_12.2, 1);
    }

    #[test]
    fn isolated_node_has_no_edges() {
        let mut b = GraphBuilder::new(2);
        b.add_clique(&[0]); // single-identifier record: no edges
        let g = b.build();
        assert_eq!(g.degree(0), 0);
        assert_eq!(g.num_nodes(), 2);
        assert_eq!(g.edges().count(), 0);
    }
}
