use anyhow::anyhow;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use std::fmt;

/// A node id is an index into the interner's reverse table. Every identifier
/// (a `(NodeKind, value)` pair) is interned exactly once and referred to by this
/// `u32` everywhere downstream, so the graph, bridge finder, and union-find all
/// operate on plain integers instead of cloning `String`s.
pub type NodeId = u32;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    AccountNumber,
    Domain,
    Abn,
    GroupId,
}

impl NodeKind {
    /// Stable lower-case label used in CSV output.
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::AccountNumber => "account_number",
            NodeKind::Domain => "domain",
            NodeKind::Abn => "abn",
            NodeKind::GroupId => "group_id",
        }
    }
}

impl TryFrom<&str> for NodeKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "account_number" => Ok(NodeKind::AccountNumber),
            "domain" => Ok(NodeKind::Domain),
            "abn" => Ok(NodeKind::Abn),
            "group_id" => Ok(NodeKind::GroupId),
            other => Err(anyhow!("Unknown node kind: {}", other)),
        }
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Assigns a dense `NodeId` to each distinct `(kind, value)` identifier and
/// remembers the mapping so ids can be turned back into identifiers for output.
#[derive(Debug, Default)]
pub struct Interner {
    lookup: FxHashMap<(NodeKind, String), NodeId>,
    nodes: Vec<(NodeKind, String)>,
}

impl Interner {
    pub fn with_capacity(capacity: usize) -> Self {
        Interner {
            lookup: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            nodes: Vec::with_capacity(capacity),
        }
    }

    /// Returns the id for `(kind, value)`, allocating a new one on first sight.
    pub fn intern(&mut self, kind: NodeKind, value: String) -> NodeId {
        let next = self.nodes.len() as NodeId;
        match self.lookup.entry((kind, value)) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                self.nodes.push(entry.key().clone());
                *entry.insert(next)
            }
        }
    }

    /// Total number of distinct identifiers seen.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    // Kept as the idiomatic companion to `len`; not currently called.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Resolves an id back to its `(kind, value)` identifier.
    pub fn resolve(&self, id: NodeId) -> &(NodeKind, String) {
        &self.nodes[id as usize]
    }
}
