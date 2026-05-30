mod bridge_finder;
mod graph;
mod input;
mod node;
mod output;
mod union_find;

use bridge_finder::find_valid_bridges;
use graph::GraphBuilder;
use node::{Interner, NodeId};
use rustc_hash::FxHashSet;
use union_find::UnionFind;

const INPUT_PATH: &str = "./data.csv";
const ENTITIES_PATH: &str = "./entities.csv";
const RECORDS_PATH: &str = "./records.csv";

fn main() -> anyhow::Result<()> {
    let records = input::parse(INPUT_PATH)?;

    // Intern every identifier to a dense id, and remember each record's ids so
    // we can build the graph and map records back to entities later.
    let mut interner = Interner::with_capacity(200_000);
    let mut record_ids = Vec::with_capacity(records.len());
    let mut record_nodes: Vec<Vec<NodeId>> = Vec::with_capacity(records.len());
    for record in &records {
        let ids = record
            .node_values()
            .into_iter()
            .map(|(kind, value)| interner.intern(kind, value))
            .collect::<Vec<NodeId>>();
        record_ids.push(record.id.clone());
        record_nodes.push(ids);
    }

    // Build the identifier graph: each record's ids form a weighted clique.
    let mut builder = GraphBuilder::new(interner.len());
    for ids in &record_nodes {
        builder.add_clique(ids);
    }
    let graph = builder.build();

    // Find the weak links to cut.
    let bridges = find_valid_bridges(&graph);
    let bridge_set: FxHashSet<(NodeId, NodeId)> = bridges.iter().copied().collect();

    // Components before cutting: union over every edge.
    let mut uf_before = UnionFind::new(interner.len());
    for (a, b, _) in graph.edges() {
        uf_before.union(a, b);
    }
    let components_before = uf_before.count();

    // Components after cutting: union over every edge except the valid bridges.
    let mut uf = UnionFind::new(interner.len());
    for (a, b, _) in graph.edges() {
        if bridge_set.contains(&(a, b)) {
            continue;
        }
        uf.union(a, b);
    }
    let components_after = uf.count();

    let isolated = (0..interner.len() as NodeId)
        .filter(|&id| graph.degree(id) == 0)
        .count();
    let skipped = record_nodes.iter().filter(|ids| ids.is_empty()).count();

    println!("records read:          {}", records.len());
    println!("  with no identifiers: {skipped} (skipped)");
    println!("identifiers (nodes):   {}", interner.len());
    println!("  isolated:            {isolated}");
    println!("valid bridges cut:     {}", bridges.len());
    println!("entities before cut:   {components_before}");
    println!("entities after cut:    {components_after}");

    output::write(
        &interner,
        &mut uf,
        &record_ids,
        &record_nodes,
        ENTITIES_PATH,
        RECORDS_PATH,
    )?;
    println!("wrote {ENTITIES_PATH} and {RECORDS_PATH}");

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::bridge_finder::find_valid_bridges;
    use crate::graph::GraphBuilder;
    use crate::node::{Interner, NodeId, NodeKind};
    use crate::union_find::UnionFind;

    /// Exercises the whole pipeline (intern → build → cut → count) on a handful
    /// of records: a chain of three records linked by shared identifiers plus a
    /// fourth single-identifier record. Confirms the two key fixes — the weak
    /// interior link is cut, and the isolated single-identifier entity is counted.
    #[test]
    fn end_to_end_links_cuts_and_counts() {
        let records = [
            vec![(NodeKind::GroupId, "g1"), (NodeKind::Abn, "a1")],
            vec![(NodeKind::Abn, "a1"), (NodeKind::Domain, "d1")], // shares a1
            vec![(NodeKind::Domain, "d1"), (NodeKind::AccountNumber, "c1")], // shares d1
            vec![(NodeKind::GroupId, "solo")],                     // single identifier
        ];

        let mut interner = Interner::with_capacity(8);
        let record_nodes: Vec<Vec<NodeId>> = records
            .iter()
            .map(|nodes| {
                nodes
                    .iter()
                    .map(|&(kind, value)| interner.intern(kind, value.to_string()))
                    .collect()
            })
            .collect();

        // Five distinct identifiers: g1, a1, d1, c1, solo.
        assert_eq!(interner.len(), 5);

        let mut builder = GraphBuilder::new(interner.len());
        for ids in &record_nodes {
            builder.add_clique(ids);
        }
        let graph = builder.build();

        // The graph is the path g1-a1-d1-c1 plus isolated `solo`; the interior
        // edge a1-d1 is the only valid bridge.
        let bridges = find_valid_bridges(&graph);
        let a1 = interner_id(&records, &record_nodes, NodeKind::Abn, "a1");
        let d1 = interner_id(&records, &record_nodes, NodeKind::Domain, "d1");
        assert_eq!(bridges, vec![(a1.min(d1), a1.max(d1))]);

        let count = |skip: bool| {
            let mut uf = UnionFind::new(interner.len());
            for (x, y, _) in graph.edges() {
                if skip && bridges.contains(&(x, y)) {
                    continue;
                }
                uf.union(x, y);
            }
            uf.count()
        };

        // Before cutting: one chain + the isolated node = 2 entities.
        assert_eq!(count(false), 2);
        // After cutting the weak link: {g1,a1}, {d1,c1}, {solo} = 3 entities.
        assert_eq!(count(true), 3);

        let isolated = (0..interner.len() as NodeId)
            .filter(|&id| graph.degree(id) == 0)
            .count();
        assert_eq!(isolated, 1);
    }

    // Finds the interned id of a given identifier by replaying the intern order.
    fn interner_id(
        records: &[Vec<(NodeKind, &str)>],
        record_nodes: &[Vec<NodeId>],
        kind: NodeKind,
        value: &str,
    ) -> NodeId {
        for (rec, ids) in records.iter().zip(record_nodes) {
            for (&(k, v), &id) in rec.iter().zip(ids) {
                if k == kind && v == value {
                    return id;
                }
            }
        }
        panic!("identifier not found");
    }
}
