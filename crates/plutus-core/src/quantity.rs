//! Signed decimal quantity. Positive = inflow to account (buy / deposit),
//! negative = outflow (sell / withdrawal). Stored as `Decimal` so fractional
//! shares (e.g. dividend-reinvest) round-trip exactly.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Quantity(pub Decimal);

impl Quantity {
    #[must_use]
    pub const fn new(value: Decimal) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    #[must_use]
    pub fn value(self) -> Decimal {
        self.0
    }

    #[must_use]
    pub fn is_positive(self) -> bool {
        self.0 > Decimal::ZERO
    }

    #[must_use]
    pub fn is_negative(self) -> bool {
        self.0 < Decimal::ZERO
    }

    #[must_use]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Add for Quantity {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Quantity {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Neg for Quantity {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}
