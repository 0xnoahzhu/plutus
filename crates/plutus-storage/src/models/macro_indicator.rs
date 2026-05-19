//! Macro / economic indicator catalog. Observations land in
//! `macro_observations`.

#[derive(Debug, toasty::Model)]
#[table = "macro_indicators"]
pub struct MacroIndicator {
    #[key]
    pub code: String, // "US_CPI" / "FOMC_RATE" / "CN_LPR_1Y" / "CN_GDP_QOQ"
    pub name: String,
    pub country: String, // ISO alpha-2 or "GLOBAL"
    pub unit: String,    // "%" / "USD" / "index" / "ratio"
    pub frequency: String, // "daily" / "monthly" / "quarterly" / "annual" / "irregular"
    pub source: String,  // "FRED" / "PBoC" / "NBS" / "agent"
    pub description: Option<String>,
}
