use anyhow::anyhow;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum NodeKind {
    AccountNumber,
    Domain,
    Abn,
    GroupId,
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

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub value: String,
}

impl Node {
    pub fn new(kind: &str, value: String) -> anyhow::Result<Self> {
        Ok(Node {
            kind: NodeKind::try_from(kind)?,
            value,
        })
    }
}
