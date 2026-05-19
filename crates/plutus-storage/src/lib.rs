//! Persistence layer for plutus. Owns toasty schema definitions, the database
//! handle, and query helpers. Returns `plutus-core` types to consumers; toasty
//! internals do not leak out.

#![recursion_limit = "512"]

pub mod db;
pub mod models;
pub mod queries;
pub mod seed;

pub use db::{Db, DbError, Result};
