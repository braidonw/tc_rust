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
    pub fn group_id(&self) -> Option<String> {
        canonicalise(&self.group_id)
    }

    pub fn account_number(&self) -> Option<String> {
        canonicalise(&self.account_number)
    }

    pub fn abn(&self) -> Option<String> {
        if let Some(str) = canonicalise(&self.abn) {
            validate_abn(str)
        } else {
            None
        }
    }

    pub fn domain(&self) -> Option<String> {
        // Discard any generic domains
        if self.generic_domain {
            return None;
        }
        canonicalise(&self.domain)
    }

    pub fn node_values(&self) -> Vec<(&str, String)> {
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

fn canonicalise(s: &str) -> Option<String> {
    if s == "NULL" || s.is_empty() {
        None
    } else {
        // Remove all whitespace
        let canonical_string = s.replace(' ', "").trim().to_lowercase();
        Some(canonical_string)
    }
}

const ABN_WEIGHTS: [usize; 11] = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];

pub fn validate_abn(s: String) -> Option<String> {
    // Discard invalid length
    if s.len() != 11 {
        return None;
    }

    let digits = s
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|d| d as usize)
        .collect::<Vec<usize>>();

    let sum = digits
        .iter()
        .zip(ABN_WEIGHTS.iter())
        .enumerate()
        .map(|(i, (d, w))| match i {
            0 => (d - 1) * w,
            _ => d * w,
        })
        .sum::<usize>();

    let remainder = sum % 89;
    if remainder == 0 {
        Some(digits.iter().map(|d| d.to_string()).collect::<String>())
    } else {
        None
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

#[cfg(test)]
mod test {

    #[test]
    fn test_valid_abn() {
        let valid_abn = "11365315258";

        assert_eq!(
            super::validate_abn(valid_abn.to_string()),
            Some(valid_abn.to_string())
        );
    }

    #[test]
    fn test_invalid_abn() {
        let invalid_abn = "11365315259";

        assert_eq!(super::validate_abn(invalid_abn.to_string()), None);
    }
}
