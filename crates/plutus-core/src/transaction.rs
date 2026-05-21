//! Transaction-related enums kept in core so storage and API layers share them.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionKind {
    Buy,
    Sell,
    Dividend,
    Fee,
    Interest,
    Deposit,
    Withdrawal,
    Fx,
    CorporateAction,
}

impl TransactionKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
            Self::Dividend => "DIVIDEND",
            Self::Fee => "FEE",
            Self::Interest => "INTEREST",
            Self::Deposit => "DEPOSIT",
            Self::Withdrawal => "WITHDRAWAL",
            Self::Fx => "FX",
            Self::CorporateAction => "CORPORATE_ACTION",
        }
    }

    /// Whether this kind affects share position. Buys/sells/corporate-actions
    /// move shares; the cash-only kinds (dividend, fee, interest, deposit,
    /// withdrawal, fx) leave the share count alone.
    #[must_use]
    pub fn moves_shares(self) -> bool {
        matches!(self, Self::Buy | Self::Sell | Self::CorporateAction)
    }
}

impl fmt::Display for TransactionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TransactionKind {
    type Err = CoreError;
    /// Case-insensitive. Accepts both `BUY` (canonical) and `buy`
    /// (lowercase) — agents have written both depending on which version
    /// of the docs they followed. The canonical written form is upper
    /// `SCREAMING_SNAKE_CASE` (matches `as_str()`); reads tolerate either.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let upper = s.to_ascii_uppercase();
        Ok(match upper.as_str() {
            "BUY" => Self::Buy,
            "SELL" => Self::Sell,
            "DIVIDEND" => Self::Dividend,
            "FEE" => Self::Fee,
            "INTEREST" => Self::Interest,
            "DEPOSIT" => Self::Deposit,
            // Common alias — the OpenAPI doc temporarily promised
            // `withdraw` instead of `withdrawal`. Keep both working.
            "WITHDRAWAL" | "WITHDRAW" => Self::Withdrawal,
            "FX" => Self::Fx,
            "CORPORATE_ACTION" => Self::CorporateAction,
            _ => return Err(CoreError::Conversion(format!("unknown TransactionKind: {s}"))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_is_case_insensitive() {
        assert_eq!("BUY".parse::<TransactionKind>().unwrap(), TransactionKind::Buy);
        assert_eq!("buy".parse::<TransactionKind>().unwrap(), TransactionKind::Buy);
        assert_eq!("Buy".parse::<TransactionKind>().unwrap(), TransactionKind::Buy);
    }

    #[test]
    fn withdraw_alias_works() {
        assert_eq!(
            "withdraw".parse::<TransactionKind>().unwrap(),
            TransactionKind::Withdrawal
        );
        assert_eq!(
            "WITHDRAWAL".parse::<TransactionKind>().unwrap(),
            TransactionKind::Withdrawal
        );
    }

    #[test]
    fn unknown_kind_errors() {
        assert!("foobar".parse::<TransactionKind>().is_err());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetClass {
    Stock,
    Etf,
    Fund,
    Bond,
    Cash,
    Other,
}

impl AssetClass {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stock => "STOCK",
            Self::Etf => "ETF",
            Self::Fund => "FUND",
            Self::Bond => "BOND",
            Self::Cash => "CASH",
            Self::Other => "OTHER",
        }
    }
}

impl FromStr for AssetClass {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "STOCK" => Self::Stock,
            "ETF" => Self::Etf,
            "FUND" => Self::Fund,
            "BOND" => Self::Bond,
            "CASH" => Self::Cash,
            "OTHER" => Self::Other,
            other => return Err(CoreError::Conversion(format!("unknown AssetClass: {other}"))),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    En,
    ZhCn,
    ZhTw,
}

impl Locale {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::ZhCn => "zh-CN",
            Self::ZhTw => "zh-TW",
        }
    }
}

impl FromStr for Locale {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "en" => Self::En,
            "zh-CN" | "zh_CN" | "zh" => Self::ZhCn,
            "zh-TW" | "zh_TW" => Self::ZhTw,
            other => return Err(CoreError::Conversion(format!("unknown Locale: {other}"))),
        })
    }
}
