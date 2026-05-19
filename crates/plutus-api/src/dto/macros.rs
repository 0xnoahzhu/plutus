use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use plutus_storage::models::{MacroIndicator, MacroObservation};

#[derive(Debug, Serialize, ToSchema)]
pub struct MacroIndicatorOut {
    pub code: String,
    pub name: String,
    pub country: String,
    pub unit: String,
    pub frequency: String,
    pub source: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct MacroIndicatorIn {
    pub code: String,
    pub name: String,
    pub country: String,
    pub unit: String,
    pub frequency: String,
    #[serde(default = "default_source")]
    pub source: String,
    pub description: Option<String>,
}

fn default_source() -> String {
    "agent".into()
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MacroObservationOut {
    pub id: i64,
    pub indicator_code: String,
    pub obs_date: String,
    #[schema(value_type = String)]
    pub value: Decimal,
    pub revised_at: Option<String>,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct MacroObservationIn {
    pub indicator_code: String,
    pub obs_date: String,
    #[schema(value_type = String)]
    pub value: Decimal,
    #[serde(default = "default_source")]
    pub source: String,
}
