//! ISO 4217 reference data. Named `CurrencyRow` to avoid colliding with
//! `plutus_core::Currency`.

#[derive(Debug, toasty::Model)]
#[table = "currencies"]
pub struct CurrencyRow {
    #[key]
    pub code: String, // ISO 4217, e.g. "USD"
    pub name: String,
    pub decimals: i32,
}
