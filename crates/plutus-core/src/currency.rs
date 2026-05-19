//! ISO 4217 currency codes. Stored as uppercased three-letter strings.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Currency(String);

impl Currency {
    pub fn new(code: impl Into<String>) -> Result<Self, CoreError> {
        let raw = code.into().to_uppercase();
        if raw.len() != 3 || !raw.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(CoreError::InvalidCurrency(raw));
        }
        Ok(Self(raw))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Number of fractional digits typically displayed for this currency.
    /// Defaults to 2 if unknown. Source: ISO 4217 / common conventions.
    #[must_use]
    pub fn display_decimals(&self) -> u32 {
        match self.0.as_str() {
            "JPY" | "KRW" | "VND" | "CLP" => 0,
            "BHD" | "JOD" | "KWD" | "OMR" | "TND" => 3,
            _ => 2,
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Currency {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Convenience constants for the currencies we touch in Phase 0.
impl Currency {
    pub fn usd() -> Self {
        Self("USD".to_string())
    }
    pub fn hkd() -> Self {
        Self("HKD".to_string())
    }
    pub fn cny() -> Self {
        Self("CNY".to_string())
    }
    pub fn sgd() -> Self {
        Self("SGD".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_validation() {
        assert!(Currency::new("USD").is_ok());
        assert!(Currency::new("usd").is_ok());
        assert!(Currency::new("US").is_err());
        assert!(Currency::new("USDD").is_err());
        assert!(Currency::new("US1").is_err());
        assert_eq!(Currency::new("usd").unwrap().as_str(), "USD");
    }

    #[test]
    fn display_decimals_known() {
        assert_eq!(Currency::usd().display_decimals(), 2);
        assert_eq!(Currency::new("JPY").unwrap().display_decimals(), 0);
        assert_eq!(Currency::new("KWD").unwrap().display_decimals(), 3);
    }
}
