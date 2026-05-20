use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{MacroIndicator, MacroObservation};

/// Definition of a macro indicator — `cpi_yoy`, `fed_funds`,
/// `gdp_qoq`, etc. Shared across users. Indicators are the parent for
/// both `macro_observations` (the time series) and `macro_events` (the
/// individual releases / decisions).
#[derive(Debug, Serialize, ToSchema)]
pub struct MacroIndicatorOut {
    /// Primary key — short code (e.g. `cpi_yoy`).
    pub code: String,
    /// Display name (e.g. "CPI Year-over-Year").
    pub name: String,
    /// ISO country code (`US` / `HK` / `CN` / `GLOBAL`).
    pub country: String,
    /// Display unit (`pct`, `bps`, `usd_billion`, `index`).
    pub unit: String,
    /// `daily` | `weekly` | `monthly` | `quarterly` | `irregular`.
    pub frequency: String,
    /// Provenance (typically the data vendor).
    pub source: String,
    /// English description of what this indicator measures and why.
    pub description: Option<String>,
}

impl From<MacroIndicator> for MacroIndicatorOut {
    fn from(i: MacroIndicator) -> Self {
        Self {
            code: i.code,
            name: i.name,
            country: i.country,
            unit: i.unit,
            frequency: i.frequency,
            source: i.source,
            description: i.description,
        }
    }
}

/// `POST /macro/indicators` body. Upserts by `code`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MacroIndicatorIn {
    /// Short code; serves as the FK target for observations / events.
    pub code: String,
    /// Display name.
    pub name: String,
    /// ISO country code or `GLOBAL`.
    pub country: String,
    /// Display unit.
    pub unit: String,
    /// `daily` | `weekly` | `monthly` | `quarterly` | `irregular`.
    pub frequency: String,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
    /// English description.
    pub description: Option<String>,
}

fn default_source() -> String {
    "agent".into()
}

/// One time-series observation for a macro indicator. The unbroken series
/// — `macro_events` carries the discrete release-level color (consensus
/// vs actual, FOMC vote, etc.).
#[derive(Debug, Serialize, ToSchema)]
pub struct MacroObservationOut {
    /// Primary key.
    pub id: i64,
    /// FK to `macro_indicators.code`.
    pub indicator_code: String,
    /// ISO date `YYYY-MM-DD` of the observation period (not the release
    /// date — see `revised_at` for that).
    pub obs_date: String,
    /// Value in the indicator's `unit`.
    #[schema(value_type = String)]
    pub value: Decimal,
    /// RFC 3339 UTC timestamp of the last revision. `null` if never
    /// revised since first publication.
    pub revised_at: Option<String>,
    /// Provenance.
    pub source: String,
}

impl From<MacroObservation> for MacroObservationOut {
    fn from(o: MacroObservation) -> Self {
        Self {
            id: o.id,
            indicator_code: o.indicator_code,
            obs_date: o.obs_date,
            value: o.value,
            revised_at: o.revised_at.map(|t| t.to_string()),
            source: o.source,
        }
    }
}

/// `POST /macro/observations` body. Upserts by `(indicator_code,
/// obs_date)` so a revision overwrites the prior value and stamps
/// `revised_at`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MacroObservationIn {
    /// FK to `macro_indicators.code`.
    pub indicator_code: String,
    /// ISO date `YYYY-MM-DD`.
    pub obs_date: String,
    /// Value in the indicator's `unit`.
    #[schema(value_type = String)]
    pub value: Decimal,
    /// Default `agent`.
    #[serde(default = "default_source")]
    pub source: String,
}
