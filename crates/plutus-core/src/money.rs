//! Money = (Decimal amount, Currency). All arithmetic enforces currency match.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::currency::Currency;
use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    #[must_use]
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    /// Add two money values; both currencies must match.
    pub fn checked_add(&self, other: &Self) -> Result<Self, CoreError> {
        if self.currency != other.currency {
            return Err(CoreError::CurrencyMismatch {
                expected: self.currency.to_string(),
                actual: other.currency.to_string(),
            });
        }
        Ok(Self {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        })
    }

    pub fn checked_sub(&self, other: &Self) -> Result<Self, CoreError> {
        if self.currency != other.currency {
            return Err(CoreError::CurrencyMismatch {
                expected: self.currency.to_string(),
                actual: other.currency.to_string(),
            });
        }
        Ok(Self {
            amount: self.amount - other.amount,
            currency: self.currency.clone(),
        })
    }

    /// Multiply by a scalar (useful for qty * price computations).
    #[must_use]
    pub fn scale(&self, factor: Decimal) -> Self {
        Self {
            amount: self.amount * factor,
            currency: self.currency.clone(),
        }
    }

    /// Convert to another currency using a fixed rate. Caller is responsible
    /// for sourcing a rate appropriate for the use case (transaction-time vs
    /// today's spot).
    #[must_use]
    pub fn convert(&self, rate: Decimal, target: Currency) -> Self {
        Self {
            amount: self.amount * rate,
            currency: target,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let decimals = self.currency.display_decimals();
        write!(f, "{} {:.*}", self.currency, decimals as usize, self.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn usd(n: Decimal) -> Money {
        Money::new(n, Currency::usd())
    }

    #[test]
    fn add_same_currency() {
        let a = usd(dec!(10.50));
        let b = usd(dec!(2.25));
        assert_eq!(a.checked_add(&b).unwrap().amount, dec!(12.75));
    }

    #[test]
    fn add_different_currency_errors() {
        let a = usd(dec!(10));
        let b = Money::new(dec!(10), Currency::hkd());
        assert!(a.checked_add(&b).is_err());
    }

    #[test]
    fn scale_qty_by_price() {
        let price = usd(dec!(185.10));
        let total = price.scale(dec!(100));
        assert_eq!(total.amount, dec!(18510.00));
    }

    #[test]
    fn display_respects_currency_decimals() {
        let jpy = Money::new(dec!(123456), Currency::new("JPY").unwrap());
        assert_eq!(format!("{jpy}"), "JPY 123456");
        let usd = usd(dec!(10));
        assert_eq!(format!("{usd}"), "USD 10.00");
    }
}
