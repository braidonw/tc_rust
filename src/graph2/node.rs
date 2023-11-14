use anyhow::anyhow;
use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
};

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum NodeKind {
    AccountNumber,
    Domain,
    Abn,
    GroupId,
}

#[derive(Debug)]
pub struct Edge<'a> {
    to: &'a Node<'a>,
    weight: usize,
}

#[derive(Debug)]
pub struct Node<'a> {
    kind: NodeKind,
    value: String,
    adjacent: Vec<&'a Node<'a>>,
    parent: Option<RefCell<&'a Node<'a>>>,
}

impl<'a> Node<'a> {
    pub fn new(kind: &str, value: &str) -> anyhow::Result<Self> {
        let kind = NodeKind::try_from(kind)?;
        let value = value.into();

        Ok(Node {
            kind,
            value,
            adjacent: Vec::new(),
            parent: None,
        })
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn add_adjacent(&mut self, node: &Node) {
        self.adjacent.push(node);
    }

    pub fn adjacent(&self) -> Vec<&Node> {
        self.adjacent.clone()
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.value == other.value
    }
}

impl<'a> Hash for Node<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.value.hash(state);
    }
}

impl<'a> Edge<'a> {
    pub fn new(to: &'a Node, weight: usize) -> Self {
        Edge { to, weight }
    }

    pub fn increment_weight(&mut self) {
        self.weight += 1;
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
