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

/// MIC codes belonging to a given ISO country (e.g. `"US"` → `["XNAS","XNYS"]`).
/// Used by handlers that accept `?country=X` to resolve membership before
/// joining against `stocks.market_code`.
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
    Ok(markets.into_iter().map(|m| m.code).collect())
}
