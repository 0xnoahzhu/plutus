//! One-shot seed data inserted on first migration. Idempotent — safe to rerun.

use plutus_core::market::Market as MarketSeed;

use crate::db::{Db, Result};
use crate::models::{Broker, CurrencyRow, Market};
use crate::queries::{macros as macros_q, sectors as sectors_q};

const CURRENCIES: &[(&str, &str, i32)] = &[
    ("USD", "US Dollar", 2),
    ("HKD", "Hong Kong Dollar", 2),
    ("CNY", "Chinese Yuan Renminbi", 2),
    ("SGD", "Singapore Dollar", 2),
    ("EUR", "Euro", 2),
    ("GBP", "Pound Sterling", 2),
    ("JPY", "Japanese Yen", 0),
];

const BROKERS: &[(&str, &str)] = &[
    ("IBKR", "Interactive Brokers"),
    ("MOOMOO_US", "Moomoo US"),
    ("FSMONE", "FSMOne"),
];

/// GICS sector top level + a useful subset of industry groups. Codes follow
/// MSCI/GICS conventions where 11 sectors are 2 digits and industry groups
/// share the first 2 digits with their parent.
const SECTORS: &[(&str, &str, Option<&str>)] = &[
    // Top-level sectors
    ("10", "Energy", None),
    ("15", "Materials", None),
    ("20", "Industrials", None),
    ("25", "Consumer Discretionary", None),
    ("30", "Consumer Staples", None),
    ("35", "Health Care", None),
    ("40", "Financials", None),
    ("45", "Information Technology", None),
    ("50", "Communication Services", None),
    ("55", "Utilities", None),
    ("60", "Real Estate", None),
    // Selected industry groups under Tech / Comm / Discretionary that are
    // commonly used; the rest can be added incrementally.
    ("4510", "Software & Services", Some("45")),
    ("4520", "Technology Hardware & Equipment", Some("45")),
    ("4530", "Semiconductors & Semiconductor Equipment", Some("45")),
    ("5010", "Telecommunication Services", Some("50")),
    ("5020", "Media & Entertainment", Some("50")),
    ("2510", "Automobiles & Components", Some("25")),
    ("2520", "Consumer Durables & Apparel", Some("25")),
    ("2550", "Consumer Discretionary Distribution & Retail", Some("25")),
    ("3510", "Health Care Equipment & Services", Some("35")),
    ("3520", "Pharmaceuticals, Biotechnology & Life Sciences", Some("35")),
    ("4010", "Banks", Some("40")),
    ("4020", "Financial Services", Some("40")),
    ("4030", "Insurance", Some("40")),
];

/// Common macro indicators. Frequency stays as a hint; the agent fills the
/// observations table.
const MACRO_INDICATORS: &[(&str, &str, &str, &str, &str)] = &[
    // (code, name, country, unit, frequency)
    ("US_CPI", "US Consumer Price Index (YoY %)", "US", "%", "monthly"),
    ("US_PPI", "US Producer Price Index (YoY %)", "US", "%", "monthly"),
    ("US_PCE", "US Personal Consumption Expenditures (YoY %)", "US", "%", "monthly"),
    ("US_NFP", "US Non-Farm Payrolls", "US", "thousand", "monthly"),
    ("US_UNEMPLOYMENT", "US Unemployment Rate", "US", "%", "monthly"),
    ("US_ISM_MANUF", "US ISM Manufacturing PMI", "US", "index", "monthly"),
    ("US_GDP_QOQ", "US Real GDP (QoQ % annualized)", "US", "%", "quarterly"),
    ("FED_FUNDS_RATE", "US Federal Funds Effective Rate", "US", "%", "daily"),
    ("FED_FUNDS_TARGET_LOWER", "Fed Funds Target Range Lower Bound", "US", "%", "irregular"),
    ("FED_FUNDS_TARGET_UPPER", "Fed Funds Target Range Upper Bound", "US", "%", "irregular"),
    ("US_10Y_YIELD", "US 10-Year Treasury Yield", "US", "%", "daily"),
    ("US_2Y_YIELD", "US 2-Year Treasury Yield", "US", "%", "daily"),
    ("VIX", "CBOE Volatility Index", "US", "index", "daily"),
    ("CN_CPI", "China CPI (YoY %)", "CN", "%", "monthly"),
    ("CN_PPI", "China PPI (YoY %)", "CN", "%", "monthly"),
    ("CN_GDP_YOY", "China Real GDP (YoY %)", "CN", "%", "quarterly"),
    ("CN_PMI_MANUF", "China Caixin Manufacturing PMI", "CN", "index", "monthly"),
    ("CN_LPR_1Y", "PBoC 1-Year Loan Prime Rate", "CN", "%", "monthly"),
    ("CN_LPR_5Y", "PBoC 5-Year Loan Prime Rate", "CN", "%", "monthly"),
    ("HK_GDP_YOY", "Hong Kong Real GDP (YoY %)", "HK", "%", "quarterly"),
    ("HK_HIBOR_3M", "Hong Kong 3-Month HIBOR", "HK", "%", "daily"),
];

pub async fn run(db: &Db) -> Result<()> {
    seed_currencies(db).await?;
    seed_markets(db).await?;
    seed_brokers(db).await?;
    seed_sectors(db).await?;
    seed_macro_indicators(db).await?;
    Ok(())
}

async fn seed_currencies(db: &Db) -> Result<()> {
    for (code, name, decimals) in CURRENCIES {
        let code_owned = code.to_string();
        let exists = db
            .with(async |d| CurrencyRow::filter_by_code(code_owned).first().exec(d).await)
            .await?;
        if exists.is_none() {
            let code = code.to_string();
            let name = name.to_string();
            let decimals = *decimals;
            db.with(async |d| {
                toasty::create!(CurrencyRow {
                    code: code,
                    name: name,
                    decimals: decimals,
                })
                .exec(d)
                .await
            })
            .await?;
        }
    }
    Ok(())
}

async fn seed_markets(db: &Db) -> Result<()> {
    for market in MarketSeed::phase0_seed() {
        let code_owned = market.code.as_str().to_string();
        let exists = db
            .with(async |d| Market::filter_by_code(code_owned).first().exec(d).await)
            .await?;
        if exists.is_none() {
            let code = market.code.as_str().to_string();
            let name = market.name.clone();
            let country = market.country.clone();
            let timezone = market.timezone.clone();
            let currency_code = market.currency.as_str().to_string();
            let default_lot_size = market.default_lot_size;
            let settlement_days = market.settlement_days;
            db.with(async |d| {
                toasty::create!(Market {
                    code: code,
                    name: name,
                    country: country,
                    timezone: timezone,
                    currency_code: currency_code,
                    default_lot_size: default_lot_size,
                    settlement_days: settlement_days,
                })
                .exec(d)
                .await
            })
            .await?;
        }
    }
    Ok(())
}

async fn seed_brokers(db: &Db) -> Result<()> {
    for (code, name) in BROKERS {
        let code_str = (*code).to_string();
        let exists = db
            .with(async |d| Broker::filter_by_code(code_str).first().exec(d).await)
            .await?;
        if exists.is_none() {
            let code = code.to_string();
            let name = name.to_string();
            db.with(async |d| {
                toasty::create!(Broker { code: code, name: name })
                    .exec(d)
                    .await
            })
            .await?;
        }
    }
    Ok(())
}

async fn seed_sectors(db: &Db) -> Result<()> {
    // Two passes so parents always exist before children (toasty doesn't enforce
    // FKs, but it keeps the data honest at app level).
    for (code, name, parent) in SECTORS.iter().filter(|(_, _, parent)| parent.is_none()) {
        sectors_q::upsert(db, code, name, *parent, "GICS").await?;
    }
    for (code, name, parent) in SECTORS.iter().filter(|(_, _, parent)| parent.is_some()) {
        sectors_q::upsert(db, code, name, *parent, "GICS").await?;
    }
    Ok(())
}

async fn seed_macro_indicators(db: &Db) -> Result<()> {
    for (code, name, country, unit, frequency) in MACRO_INDICATORS {
        macros_q::upsert_indicator(
            db,
            macros_q::NewIndicator {
                code,
                name,
                country,
                unit,
                frequency,
                source: "agent",
                description: None,
            },
        )
        .await?;
    }
    Ok(())
}
