#![allow(dead_code)]
use csv::Reader;
use serde::{Deserialize, Deserializer};
use std::fs::File;

#[derive(Deserialize, Debug)]
pub struct Record {
    #[serde(rename = "Id")]
    id: String,
    pub group_id: String,
    pub account_number: String,
    pub abn: String,
    #[serde(rename = "email_domain_name")]
    pub domain: String,
    #[serde(deserialize_with = "bool_from_string")]
    pub generic_domain: bool,
}

// Custom serde deserializer function to handle bool values
// that are represented as 1s and 0s in the source data
pub fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_ref() {
        "1" => Ok(true),
        "0" => Ok(false),
        other => Err(serde::de::Error::custom(format!(
            "Expected 1 or 0, got {}",
            other
        ))),
    }
}

impl Record {
    pub fn group_id(&self) -> Option<&str> {
        canonicalise(&self.group_id)
    }

    pub fn account_number(&self) -> Option<&str> {
        canonicalise(&self.account_number)
    }

    pub fn abn(&self) -> Option<&str> {
        canonicalise(&self.abn)
    }

    pub fn domain(&self) -> Option<&str> {
        // Discard any generic domains
        if self.generic_domain {
            return None;
        }
        canonicalise(&self.domain)
    }

    pub fn node_values(&self) -> Vec<(&str, &str)> {
        let mut values = vec![];
        if let Some(group_id) = self.group_id() {
            values.push(("group_id", group_id));
        }
        if let Some(account_number) = self.account_number() {
            values.push(("account_number", account_number));
        }
        if let Some(abn) = self.abn() {
            values.push(("abn", abn));
        }
        if let Some(domain) = self.domain() {
            values.push(("domain", domain));
        }
        values
    }
}

fn canonicalise(s: &str) -> Option<&str> {
    if s == "NULL" || s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn parse(filename: &str) -> anyhow::Result<Vec<Record>> {
    let file = File::open(filename)?;
    let mut reader = Reader::from_reader(file);

    let mut records = vec![];
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}
