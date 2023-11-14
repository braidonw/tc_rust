use anyhow::anyhow;
use std::rc::Rc;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Node {
    kind: NodeKind,
    value: Rc<str>,
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum NodeKind {
    AccountNumber,
    Domain,
    Abn,
    GroupId,
}

impl Node {
    pub fn new(kind: &str, value: &str) -> anyhow::Result<Self> {
        if value == "NULL" || value.is_empty() {
            return Err(anyhow!("Invalid node value: {}", value));
        };

        let kind = NodeKind::try_from(kind)?;
        let value = value.into();

        Ok(Node { kind, value })
    }

    pub fn value(&self) -> &str {
        &self.value
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
