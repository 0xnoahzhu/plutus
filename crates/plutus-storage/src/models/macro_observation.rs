//! One data point of a macro indicator. Natural key (indicator_code, obs_date)
//! enforced at the app layer.

use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "macro_observations"]
pub struct MacroObservation {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub indicator_code: String,
    pub obs_date: String, // ISO date "YYYY-MM-DD"
    pub value: Decimal,
    pub revised_at: Option<jiff::Timestamp>, // null until a revision lands
    pub source: String,
    pub created_at: jiff::Timestamp,
}
