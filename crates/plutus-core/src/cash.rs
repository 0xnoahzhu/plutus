//! Cash-flow effect of a transaction, in the account's base currency.
//!
//! The sibling of [`crate::cost_basis`]: that module answers "what do I
//! own", this one answers "what did it do to my cash". Together they
//! make up total assets — market value of the open positions plus the
//! cash sitting beside them.
//!
//! Like `cost_basis`, these are pure functions on plain inputs; callers
//! map their DB rows into [`CashLot`] first.

use rust_decimal::Decimal;

use crate::transaction::TransactionKind;

/// The fields of a transaction that move cash. Amounts are in the row's
/// own currencies; `fx_to_base` converts them to the account's base
/// currency at execution time, the same rate `cost_basis` uses.
#[derive(Debug, Clone)]
pub struct CashLot {
    pub kind: TransactionKind,
    /// Shares for trades, cash amount for the cash-only kinds.
    pub quantity: Decimal,
    /// Per-share price, or `1` for cash entries.
    pub price: Decimal,
    pub commission: Decimal,
    pub tax: Decimal,
    pub fx_to_base: Decimal,
}

/// Signed change to the cash balance, in base currency. Positive means
/// cash came in.
///
/// Costs are always subtracted regardless of direction — commission and
/// tax leave the account whether you were buying or selling.
///
/// Two kinds contribute nothing. `FX` is a pure currency swap: it moves
/// value between two currency sleeves without changing what the account
/// is worth, and this ledger has no per-sleeve balances to move it
/// between, so counting it either way would invent money. Corporate
/// actions (splits, stock dividends) change the share count, not cash —
/// a cash-settled one should be posted as a separate `DIVIDEND`.
#[must_use]
pub fn cash_delta_base(lot: &CashLot) -> Decimal {
    let costs = (lot.commission + lot.tax) * lot.fx_to_base;
    let gross = lot.quantity.abs() * lot.price * lot.fx_to_base;

    let signed_gross = match lot.kind {
        // A negative quantity is the documented way to write a
        // correcting entry, so it flips the cash direction too — an
        // "unbought" 10 shares puts money back. Same reading
        // `cost_basis` applies to the share side.
        TransactionKind::Buy => {
            if lot.quantity < Decimal::ZERO {
                gross
            } else {
                -gross
            }
        }
        TransactionKind::Sell => {
            if lot.quantity < Decimal::ZERO {
                -gross
            } else {
                gross
            }
        }
        TransactionKind::Deposit | TransactionKind::Dividend | TransactionKind::Interest => {
            if lot.quantity < Decimal::ZERO {
                -gross
            } else {
                gross
            }
        }
        TransactionKind::Withdrawal | TransactionKind::Fee => {
            if lot.quantity < Decimal::ZERO {
                gross
            } else {
                -gross
            }
        }
        TransactionKind::Fx | TransactionKind::CorporateAction => Decimal::ZERO,
    };

    signed_gross - costs
}

/// Sum the cash effect of every lot. Convenience over `map().sum()` so
/// callers don't re-derive the fold.
#[must_use]
pub fn cash_total_base(lots: &[CashLot]) -> Decimal {
    lots.iter().map(cash_delta_base).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn lot(kind: TransactionKind, quantity: Decimal, price: Decimal) -> CashLot {
        CashLot {
            kind,
            quantity,
            price,
            commission: Decimal::ZERO,
            tax: Decimal::ZERO,
            fx_to_base: dec!(1),
        }
    }

    #[test]
    fn buy_takes_cash_out_including_costs() {
        let mut l = lot(TransactionKind::Buy, dec!(100), dec!(10));
        l.commission = dec!(1.05);
        l.tax = dec!(0.5);
        // -1000 spent, -1.55 costs.
        assert_eq!(cash_delta_base(&l), dec!(-1001.55));
    }

    #[test]
    fn sell_brings_cash_in_minus_costs() {
        let mut l = lot(TransactionKind::Sell, dec!(60), dec!(215.75));
        l.commission = dec!(1.10);
        l.tax = dec!(3.20);
        // +12945 proceeds, -4.30 costs.
        assert_eq!(cash_delta_base(&l), dec!(12940.70));
    }

    #[test]
    fn deposit_and_withdrawal_are_mirror_images() {
        let d = lot(TransactionKind::Deposit, dec!(20000), dec!(1));
        let w = lot(TransactionKind::Withdrawal, dec!(20000), dec!(1));
        assert_eq!(cash_delta_base(&d), dec!(20000));
        assert_eq!(cash_delta_base(&w), dec!(-20000));
    }

    #[test]
    fn dividend_is_net_of_withholding() {
        let mut l = lot(TransactionKind::Dividend, dec!(36), dec!(1));
        l.tax = dec!(5.40);
        assert_eq!(cash_delta_base(&l), dec!(30.60));
    }

    #[test]
    fn fee_and_interest_move_opposite_ways() {
        assert_eq!(cash_delta_base(&lot(TransactionKind::Fee, dec!(12), dec!(1))), dec!(-12));
        assert_eq!(
            cash_delta_base(&lot(TransactionKind::Interest, dec!(3.5), dec!(1))),
            dec!(3.5)
        );
    }

    #[test]
    fn fx_and_corporate_action_are_cash_neutral() {
        // A currency swap doesn't change what the account is worth, and
        // there are no per-currency sleeves here to move value between.
        assert_eq!(cash_delta_base(&lot(TransactionKind::Fx, dec!(5000), dec!(1))), dec!(0));
        // A split changes share count, not cash.
        assert_eq!(
            cash_delta_base(&lot(TransactionKind::CorporateAction, dec!(100), dec!(0))),
            dec!(0)
        );
    }

    #[test]
    fn fx_still_charges_its_costs() {
        // The conversion itself is neutral, but the broker's fee on it
        // is real money leaving the account.
        let mut l = lot(TransactionKind::Fx, dec!(5000), dec!(1));
        l.commission = dec!(8);
        assert_eq!(cash_delta_base(&l), dec!(-8));
    }

    #[test]
    fn negative_quantity_reverses_the_direction() {
        // The compensating-entry convention: a BUY of -10 undoes a buy,
        // so cash comes back.
        let undo_buy = lot(TransactionKind::Buy, dec!(-10), dec!(50));
        assert_eq!(cash_delta_base(&undo_buy), dec!(500));
        let undo_sell = lot(TransactionKind::Sell, dec!(-10), dec!(50));
        assert_eq!(cash_delta_base(&undo_sell), dec!(-500));
    }

    #[test]
    fn fx_rate_converts_everything_including_costs() {
        let mut l = lot(TransactionKind::Buy, dec!(500), dec!(48.20));
        l.commission = dec!(88);
        l.tax = dec!(12);
        l.fx_to_base = dec!(0.1282);
        // gross 24100 HKD -> 3089.62 base; costs 100 HKD -> 12.82 base.
        assert_eq!(cash_delta_base(&l), dec!(-3102.44));
    }

    #[test]
    fn total_folds_a_realistic_history() {
        let mut buy = lot(TransactionKind::Buy, dec!(100), dec!(172.40));
        buy.commission = dec!(1.05);
        let mut sell = lot(TransactionKind::Sell, dec!(60), dec!(215.75));
        sell.commission = dec!(1.10);
        sell.tax = dec!(3.20);
        let lots = vec![
            lot(TransactionKind::Deposit, dec!(20000), dec!(1)),
            buy,
            sell,
        ];
        // 20000 - 17241.05 + 12940.70
        assert_eq!(cash_total_base(&lots), dec!(15699.65));
    }
}
