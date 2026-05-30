use crate::node::{Interner, NodeId};
use crate::union_find::UnionFind;
use rustc_hash::FxHashMap;

/// Writes the resolved clusters to two CSV files using the post-bridge-removal
/// union-find, in which every node's root is its entity id.
///
/// - `entities_path`: one row per identifier — `entity_id, kind, value`.
/// - `records_path`: one row per record — `record_id, entity_id, split`, where
///   the record is assigned to the entity holding the majority of its
///   identifiers (ties broken by smallest `entity_id`) and `split` is true when
///   a record's identifiers span more than one entity (its linking edge was cut
///   as a weak bridge).
///
/// Records with no valid identifiers are skipped (they cannot be resolved).
pub fn write(
    interner: &Interner,
    uf: &mut UnionFind,
    record_ids: &[String],
    record_nodes: &[Vec<NodeId>],
    entities_path: &str,
    records_path: &str,
) -> anyhow::Result<()> {
    write_entities(interner, uf, entities_path)?;
    write_records(uf, record_ids, record_nodes, records_path)?;
    Ok(())
}

fn write_entities(interner: &Interner, uf: &mut UnionFind, path: &str) -> anyhow::Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(["entity_id", "kind", "value"])?;
    for id in 0..interner.len() as NodeId {
        let entity_id = uf.find(id);
        let (kind, value) = interner.resolve(id);
        writer.write_record([&entity_id.to_string(), kind.as_str(), value])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_records(
    uf: &mut UnionFind,
    record_ids: &[String],
    record_nodes: &[Vec<NodeId>],
    path: &str,
) -> anyhow::Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(["record_id", "entity_id", "split"])?;

    for (record_id, nodes) in record_ids.iter().zip(record_nodes) {
        // Skip records with no resolvable identifiers.
        if nodes.is_empty() {
            continue;
        }

        // Tally which entity each identifier belongs to.
        let mut counts: FxHashMap<NodeId, u32> = FxHashMap::default();
        for &node in nodes {
            *counts.entry(uf.find(node)).or_insert(0) += 1;
        }

        // Majority entity, ties broken by the smallest entity id.
        let entity_id = counts
            .iter()
            .max_by(|(id_a, count_a), (id_b, count_b)| count_a.cmp(count_b).then(id_b.cmp(id_a)))
            .map(|(&id, _)| id)
            .expect("record has at least one identifier");

        let split = counts.len() > 1;
        writer.write_record([record_id, &entity_id.to_string(), &split.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}
