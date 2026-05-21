use crate::db::{Db, Result};
use crate::models::Market;

pub async fn list(db: &Db) -> Result<Vec<Market>> {
    db.with(async |d| Market::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, code: &str) -> Result<Option<Market>> {
    let code = code.to_string();
    db.with(async |d| Market::filter_by_code(code).first().exec(d).await)
        .await
        .map_err(Into::into)
}

/// Market codes belonging to a given ISO country (e.g. `"US"` →
/// `["XNAS","XNYS","us","us_etf","us_adr"]`). Used by handlers that
/// accept `?country=X` to resolve membership before joining against
/// `stocks.market_code`.
///
/// Two conventions coexist in the data: the canonical MIC codes seeded
/// into the `markets` reference table (`XNAS`, `XHKG`, ...) and the
/// lowercase pseudo-codes the `stocks` table actually stores (`us`,
/// `us_etf`, `hk`, `cn_a`, ...). The result includes both so callers
/// don't have to care which convention any individual row was written
/// with. The pseudo-codes are hardcoded here — they're an implicit part
/// of the schema, not data the agent populates.
pub async fn list_codes_by_country(db: &Db, country: &str) -> Result<Vec<String>> {
    let country_owned = country.to_string();
    let markets: Vec<Market> = db
        .with(async |d| {
            Market::all()
                .filter(Market::fields().country().eq(&country_owned))
                .exec(d)
                .await
        })
        .await?;
    let mut codes: Vec<String> = markets.into_iter().map(|m| m.code).collect();
    for pseudo in pseudo_codes_for_country(country) {
        codes.push((*pseudo).to_string());
    }
    Ok(codes)
}

/// The lowercase pseudo-codes the `stocks` table uses. Hardcoded —
/// these are part of the schema convention, not user-tunable data.
fn pseudo_codes_for_country(country: &str) -> &'static [&'static str] {
    match country {
        "US" => &["us", "us_etf", "us_adr"],
        "HK" => &["hk", "hk_etf"],
        "CN" => &["cn_a", "cn_etf"],
        _ => &[],
    }
}
