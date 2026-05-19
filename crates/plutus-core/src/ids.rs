//! Strongly-typed identifiers. All IDs are `i64` keyed at the DB layer; we wrap
//! them so a `StockId` can never be passed where a `TransactionId` is expected.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub i64);

        impl $name {
            #[must_use]
            pub const fn new(v: i64) -> Self {
                Self(v)
            }
            #[must_use]
            pub const fn value(self) -> i64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<i64> for $name {
            fn from(v: i64) -> Self {
                Self(v)
            }
        }
    };
}

id_newtype!(StockId);
id_newtype!(BrokerId);
id_newtype!(AccountId);
id_newtype!(WatchlistId);
id_newtype!(TransactionId);
id_newtype!(AuditLogId);

/// ISO 10383 Market Identifier Code, e.g. `XNAS`, `XHKG`. Stored as the
/// uppercased four-letter string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MarketCode(String);

impl MarketCode {
    pub fn new(code: impl Into<String>) -> Result<Self, crate::CoreError> {
        let raw = code.into().to_uppercase();
        if raw.len() != 4 || !raw.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(crate::CoreError::InvalidMarket(raw));
        }
        Ok(Self(raw))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MarketCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_code_rejects_garbage() {
        assert!(MarketCode::new("XX").is_err());
        assert!(MarketCode::new("12345").is_err());
        assert!(MarketCode::new("XNA1").is_err());
        assert!(MarketCode::new("XNAS").is_ok());
        assert_eq!(MarketCode::new("xnas").unwrap().as_str(), "XNAS");
    }

    #[test]
    fn id_newtypes_are_distinct() {
        let s = StockId::new(1);
        let t = TransactionId::new(1);
        // Types differ; can't compare directly, but value() should match.
        assert_eq!(s.value(), t.value());
    }
}
