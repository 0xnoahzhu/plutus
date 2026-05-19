//! Data provenance. Every writable row carries a `source` + optional metadata
//! so we can tell apart user edits, agent writes, and bulk imports.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Manual,
    Agent,
    Import,
    External(String),
}

impl Source {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Manual => "manual",
            Self::Agent => "agent",
            Self::Import => "import",
            Self::External(s) => s.as_str(),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Source {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "manual" => Self::Manual,
            "agent" => Self::Agent,
            "import" => Self::Import,
            other if !other.is_empty() => Self::External(other.to_string()),
            _ => return Err(CoreError::Conversion("empty source".into())),
        })
    }
}

pub type SourceMetadata = serde_json::Value;
