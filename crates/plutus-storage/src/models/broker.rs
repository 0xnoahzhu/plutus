//! Brokerage platforms (IBKR, Moomoo US, FSMOne). Phase 0 seeds these three.

#[derive(Debug, toasty::Model)]
#[table = "brokers"]
pub struct Broker {
    #[key]
    #[auto]
    pub id: i64,
    #[unique]
    pub code: String, // "IBKR", "MOOMOO_US", "FSMONE"
    pub name: String,
}
