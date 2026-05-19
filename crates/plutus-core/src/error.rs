use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid currency code: {0}")]
    InvalidCurrency(String),
    #[error("invalid market code: {0}")]
    InvalidMarket(String),
    #[error("invalid quantity: {0}")]
    InvalidQuantity(String),
    #[error("invalid money amount: {0}")]
    InvalidMoney(String),
    #[error("currency mismatch: expected {expected}, got {actual}")]
    CurrencyMismatch { expected: String, actual: String },
    #[error("conversion error: {0}")]
    Conversion(String),
}
