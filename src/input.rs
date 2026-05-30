use crate::node::NodeKind;
use csv::Reader;
use serde::{Deserialize, Deserializer};
use std::fs::File;

#[derive(Deserialize, Debug)]
pub struct Record {
    #[serde(rename = "Id")]
    pub id: String,
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
        canonicalise(&self.abn).and_then(validate_abn)
    }

    pub fn domain(&self) -> Option<String> {
        // Discard any generic domains
        if self.generic_domain {
            return None;
        }
        canonicalise(&self.domain)
    }

    /// The valid identifier nodes this record asserts, as `(kind, value)` pairs.
    pub fn node_values(&self) -> Vec<(NodeKind, String)> {
        let mut values = Vec::with_capacity(4);
        if let Some(group_id) = self.group_id() {
            values.push((NodeKind::GroupId, group_id));
        }
        if let Some(account_number) = self.account_number() {
            values.push((NodeKind::AccountNumber, account_number));
        }
        if let Some(abn) = self.abn() {
            values.push((NodeKind::Abn, abn));
        }
        if let Some(domain) = self.domain() {
            values.push((NodeKind::Domain, domain));
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
        if canonical_string.is_empty() {
            None
        } else {
            Some(canonical_string)
        }
    }
}

const ABN_WEIGHTS: [u32; 11] = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];

/// Validates an 11-digit ABN against its modulus-89 checksum, returning the
/// canonical digit string when valid. Rejects (rather than panics on) any
/// non-digit characters or a leading zero.
pub fn validate_abn(s: String) -> Option<String> {
    if s.len() != 11 {
        return None;
    }

    // `collect::<Option<_>>` yields `None` if any character is not a digit, so a
    // success guarantees exactly 11 digits to zip against the weights.
    let digits = s
        .chars()
        .map(|c| c.to_digit(10))
        .collect::<Option<Vec<u32>>>()?;

    // The checksum subtracts 1 from the first digit; a leading zero is invalid.
    if digits[0] == 0 {
        return None;
    }

    let sum: u32 = digits
        .iter()
        .zip(ABN_WEIGHTS)
        .enumerate()
        .map(|(i, (&d, w))| if i == 0 { (d - 1) * w } else { d * w })
        .sum();

    if sum % 89 == 0 {
        Some(s)
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
    use super::validate_abn;

    #[test]
    fn test_valid_abn() {
        let valid_abn = "11365315258";
        assert_eq!(
            validate_abn(valid_abn.to_string()),
            Some(valid_abn.to_string())
        );
    }

    #[test]
    fn test_invalid_abn() {
        let invalid_abn = "11365315259";
        assert_eq!(validate_abn(invalid_abn.to_string()), None);
    }

    #[test]
    fn test_abn_wrong_length() {
        assert_eq!(validate_abn("123".to_string()), None);
        assert_eq!(validate_abn("113653152580".to_string()), None);
    }

    #[test]
    fn test_abn_non_digit_does_not_panic() {
        // An 11-char string with a letter must be rejected, not panic.
        assert_eq!(validate_abn("1136531525x".to_string()), None);
    }

    #[test]
    fn test_abn_leading_zero_does_not_panic() {
        // First digit 0 would underflow `(d - 1)` if unchecked.
        assert_eq!(validate_abn("01365315258".to_string()), None);
    }
}
