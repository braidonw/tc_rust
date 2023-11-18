pub trait UnionFind<T> {
    fn find(&mut self, node: &T) -> T;
    fn union(&mut self, node1: &T, node2: &T);
}
