use crate::node::NodeId;

/// Disjoint-set forest over dense node ids. Initialised with every node as its
/// own singleton component (so isolated identifiers are counted), then merged
/// one edge at a time. `count` tracks the number of components and only drops on
/// a merge that actually joins two distinct sets.
#[derive(Debug)]
pub struct UnionFind {
    parent: Vec<NodeId>,
    size: Vec<u32>,
    count: usize,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n as NodeId).collect(),
            size: vec![1; n],
            count: n,
        }
    }

    /// Number of disjoint components.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns the representative root of `x`, compressing the path with
    /// path-halving (point every other node at its grandparent) as it climbs.
    pub fn find(&mut self, mut x: NodeId) -> NodeId {
        while self.parent[x as usize] != x {
            let grandparent = self.parent[self.parent[x as usize] as usize];
            self.parent[x as usize] = grandparent;
            x = grandparent;
        }
        x
    }

    /// Merges the sets containing `a` and `b`, attaching the smaller tree under
    /// the larger (union by size). Returns whether a merge actually happened.
    pub fn union(&mut self, a: NodeId, b: NodeId) -> bool {
        let mut ra = self.find(a);
        let mut rb = self.find(b);
        if ra == rb {
            return false;
        }

        // Attach the smaller-sized root under the larger one.
        if self.size[ra as usize] < self.size[rb as usize] {
            std::mem::swap(&mut ra, &mut rb);
        }
        self.parent[rb as usize] = ra;
        self.size[ra as usize] += self.size[rb as usize];
        self.count -= 1;
        true
    }
}

#[cfg(test)]
mod test {
    use super::UnionFind;

    #[test]
    fn singletons_start_disjoint() {
        let uf = UnionFind::new(5);
        assert_eq!(uf.count(), 5);
    }

    #[test]
    fn union_reduces_count_only_on_real_merge() {
        let mut uf = UnionFind::new(4);
        assert!(uf.union(0, 1));
        assert_eq!(uf.count(), 3);
        // Re-unioning the same set must not change the count.
        assert!(!uf.union(0, 1));
        assert_eq!(uf.count(), 3);
        assert!(uf.union(2, 3));
        assert!(uf.union(1, 2));
        assert_eq!(uf.count(), 1);
    }

    #[test]
    fn find_agrees_within_a_set() {
        let mut uf = UnionFind::new(4);
        uf.union(0, 1);
        uf.union(1, 2);
        let root = uf.find(0);
        assert_eq!(uf.find(1), root);
        assert_eq!(uf.find(2), root);
        assert_ne!(uf.find(3), root);
    }
}
