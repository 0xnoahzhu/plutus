//! Core types and pure business logic for plutus.
//!
//! This crate has no I/O, no database, no HTTP. It defines value types
//! (Money, Currency, Quantity, ...), domain enums, audit primitives, and
//! cost-basis algorithms operating on plain transaction slices.

#![allow(clippy::module_name_repetitions)]

pub mod audit;
pub mod cost_basis;
pub mod currency;
pub mod error;
pub mod ids;
pub mod market;
pub mod money;
pub mod provenance;
pub mod quantity;
pub mod transaction;

pub use audit::{Actor, ActorKind, AuditAction};
pub use currency::Currency;
pub use error::CoreError;
pub use ids::{
    AccountId, AuditLogId, BrokerId, MarketCode, StockId, TransactionId, WatchlistId,
};
pub use market::Market;
pub use money::Money;
pub use provenance::{Source, SourceMetadata};
pub use quantity::Quantity;
pub use transaction::{AssetClass, Locale, TransactionKind};
