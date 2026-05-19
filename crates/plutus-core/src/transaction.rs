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
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "BUY" => Self::Buy,
            "SELL" => Self::Sell,
            "DIVIDEND" => Self::Dividend,
            "FEE" => Self::Fee,
            "INTEREST" => Self::Interest,
            "DEPOSIT" => Self::Deposit,
            "WITHDRAWAL" => Self::Withdrawal,
            "FX" => Self::Fx,
            "CORPORATE_ACTION" => Self::CorporateAction,
            other => return Err(CoreError::Conversion(format!("unknown TransactionKind: {other}"))),
        })
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
