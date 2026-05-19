//! Market reference metadata. Each `Market` instance corresponds to a row in
//! the `markets` table.

use serde::{Deserialize, Serialize};

use crate::currency::Currency;
use crate::ids::MarketCode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Market {
    pub code: MarketCode,
    pub name: String,
    pub country: String,
    pub timezone: String,
    pub currency: Currency,
    pub default_lot_size: i32,
    pub settlement_days: i32,
}

impl Market {
    /// Seed data for Phase 0 — US (NASDAQ + NYSE), HK (HKEX), CN (Shanghai + Shenzhen).
    #[must_use]
    pub fn phase0_seed() -> Vec<Self> {
        vec![
            Self {
                code: MarketCode::new("XNAS").unwrap(),
                name: "NASDAQ".into(),
                country: "US".into(),
                timezone: "America/New_York".into(),
                currency: Currency::usd(),
                default_lot_size: 1,
                settlement_days: 1,
            },
            Self {
                code: MarketCode::new("XNYS").unwrap(),
                name: "New York Stock Exchange".into(),
                country: "US".into(),
                timezone: "America/New_York".into(),
                currency: Currency::usd(),
                default_lot_size: 1,
                settlement_days: 1,
            },
            Self {
                code: MarketCode::new("XHKG").unwrap(),
                name: "Hong Kong Stock Exchange".into(),
                country: "HK".into(),
                timezone: "Asia/Hong_Kong".into(),
                currency: Currency::hkd(),
                default_lot_size: 100,
                settlement_days: 2,
            },
            Self {
                code: MarketCode::new("XSHG").unwrap(),
                name: "Shanghai Stock Exchange".into(),
                country: "CN".into(),
                timezone: "Asia/Shanghai".into(),
                currency: Currency::cny(),
                default_lot_size: 100,
                settlement_days: 1,
            },
            Self {
                code: MarketCode::new("XSHE").unwrap(),
                name: "Shenzhen Stock Exchange".into(),
                country: "CN".into(),
                timezone: "Asia/Shanghai".into(),
                currency: Currency::cny(),
                default_lot_size: 100,
                settlement_days: 1,
            },
        ]
    }
}
