//! Domain identifiers.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A calibre book id, carried as a string and ordered numerically.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct WorkId(pub String);

impl WorkId {
    pub fn numeric(&self) -> Option<u64> {
        self.0.parse().ok()
    }
}

impl fmt::Display for WorkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialOrd for WorkId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorkId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.numeric(), other.numeric()) {
            (Some(left), Some(right)) => left.cmp(&right),
            _ => self.0.cmp(&other.0),
        }
    }
}

/// `calibre:<work_id>:<format>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct EditionId(pub String);

impl EditionId {
    pub fn new(work_id: &WorkId, format: &str) -> Self {
        Self(format!("calibre:{}:{}", work_id.0, format.to_ascii_lowercase()))
    }
}

impl fmt::Display for EditionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Hex sha256 of the edition bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct ContentHash(pub String);

/// Hex sha256 over content hash, chapter, passage ordinal and offsets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct PassageUid(pub String);
