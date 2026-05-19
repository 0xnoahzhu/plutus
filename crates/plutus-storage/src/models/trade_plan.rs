//! Trade plan header. One row per per-user, per-stock plan; each plan
//! carries N price points in `trade_plan_levels`. Multiple active plans
//! for the same stock are allowed (e.g. a "bullish" plan and a "defensive"
//! plan can coexist).
//!
//! Plans are advisory — the system records the user's intent and lets them
//! flip a level's status to `triggered` when they actually execute. There's
//! no OHLCV-cross auto-trigger; the user is the decision-maker.

#[derive(Debug, toasty::Model)]
#[table = "trade_plans"]
pub struct TradePlan {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub user_id: i64,
    #[index]
    pub stock_id: i64,
    /// Plan-level rationale — why this set of price points exists.
    /// Plain text, single-language; multi-locale upgrade is a refactor for
    /// later if the need arises.
    pub rationale: Option<String>,
    /// `active` while the plan is being followed, `closed` once retired.
    /// Closing a plan keeps the row + levels around for history; deleting
    /// the row cascades to remove every level under it.
    pub status: String,
    pub created_at: jiff::Timestamp,
    pub updated_at: jiff::Timestamp,
}
