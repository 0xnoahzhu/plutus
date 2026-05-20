//! Pending-order CRUD. Per-user, per-account, per-stock — same shape as
//! `trade_plan_levels` queries but flat (no parent header).
//!
//! Status flips also stamp/clear the matching `*_at` timestamps so the
//! UI never has to keep them in sync manually: `open → filled` sets
//! `filled_at = now()`, `open → cancelled` sets `cancelled_at`, and
//! switching back to `open` clears both. `expired` is passive — caller
//! sets it without a fresh timestamp because the meaningful moment was
//! `expires_at`.

use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::PendingOrder;

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub account_id: Option<i64>,
    pub stock_id: Option<i64>,
    pub status: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<PendingOrder>> {
    let user_id = filter.user_id;
    let status_filter = filter.status.map(str::to_string);
    let rows = db
        .with(async |d| PendingOrder::all().exec(d).await)
        .await?;
    let mut rows: Vec<PendingOrder> = rows
        .into_iter()
        .filter(|r| r.user_id == user_id)
        .filter(|r| filter.account_id.map_or(true, |a| r.account_id == a))
        .filter(|r| filter.stock_id.map_or(true, |s| r.stock_id == s))
        .filter(|r| status_filter.as_deref().map_or(true, |s| r.status == s))
        .collect();
    // Most-recently-placed first — matches "what's live right now" reads.
    rows.sort_by(|a, b| b.placed_at.cmp(&a.placed_at));
    Ok(rows)
}

pub async fn get(db: &Db, user_id: i64, id: i64) -> Result<PendingOrder> {
    let row = db
        .with(async |d| PendingOrder::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewOrder<'a> {
    pub user_id: i64,
    pub account_id: i64,
    pub stock_id: i64,
    pub trade_plan_level_id: Option<i64>,
    pub side: &'a str,
    pub order_type: &'a str,
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub quantity: Decimal,
    pub time_in_force: &'a str,
    pub expires_at: Option<jiff::Timestamp>,
    pub broker_order_ref: Option<&'a str>,
    pub notes: Option<&'a str>,
    /// Optional override — defaults to `now()` when None. Lets a user
    /// backfill an order they placed earlier in the day.
    pub placed_at: Option<jiff::Timestamp>,
}

pub async fn create(db: &Db, input: NewOrder<'_>) -> Result<PendingOrder> {
    let now = jiff::Timestamp::now();
    let user_id = input.user_id;
    let account_id = input.account_id;
    let stock_id = input.stock_id;
    let trade_plan_level_id = input.trade_plan_level_id;
    let side = input.side.to_string();
    let order_type = input.order_type.to_string();
    let limit_price = input.limit_price;
    let stop_price = input.stop_price;
    let quantity = input.quantity;
    let time_in_force = input.time_in_force.to_string();
    let expires_at = input.expires_at;
    let broker_order_ref = input.broker_order_ref.map(str::to_string);
    let notes = input.notes.map(str::to_string);
    let placed_at = input.placed_at.unwrap_or(now);
    let row = db
        .with(async |d| {
            toasty::create!(PendingOrder {
                user_id: user_id,
                account_id: account_id,
                stock_id: stock_id,
                trade_plan_level_id: trade_plan_level_id,
                side: side,
                order_type: order_type,
                limit_price: limit_price,
                stop_price: stop_price,
                quantity: quantity,
                time_in_force: time_in_force,
                expires_at: expires_at,
                status: "open".to_string(),
                placed_at: placed_at,
                filled_at: None::<jiff::Timestamp>,
                cancelled_at: None::<jiff::Timestamp>,
                broker_order_ref: broker_order_ref,
                notes: notes,
                created_at: now,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub struct OrderPatch<'a> {
    pub account_id: Option<i64>,
    pub side: Option<&'a str>,
    pub order_type: Option<&'a str>,
    pub limit_price: Option<Option<Decimal>>,
    pub stop_price: Option<Option<Decimal>>,
    pub quantity: Option<Decimal>,
    pub time_in_force: Option<&'a str>,
    pub expires_at: Option<Option<jiff::Timestamp>>,
    pub broker_order_ref: Option<Option<&'a str>>,
    pub notes: Option<Option<&'a str>>,
    /// `open` / `filled` / `cancelled` / `expired`. Flipping to `filled`
    /// or `cancelled` stamps the matching `*_at` server-side; switching
    /// back to `open` clears both.
    pub status: Option<&'a str>,
}

pub async fn update(
    db: &Db,
    user_id: i64,
    id: i64,
    patch: OrderPatch<'_>,
) -> Result<PendingOrder> {
    let mut row = get(db, user_id, id).await?;
    let now = jiff::Timestamp::now();
    db.with(async |d| {
        let mut q = row.update();
        if let Some(account_id) = patch.account_id {
            q = q.account_id(account_id);
        }
        if let Some(side) = patch.side {
            q = q.side(side.to_string());
        }
        if let Some(order_type) = patch.order_type {
            q = q.order_type(order_type.to_string());
        }
        if let Some(limit_price) = patch.limit_price {
            q = q.limit_price(limit_price);
        }
        if let Some(stop_price) = patch.stop_price {
            q = q.stop_price(stop_price);
        }
        if let Some(quantity) = patch.quantity {
            q = q.quantity(quantity);
        }
        if let Some(time_in_force) = patch.time_in_force {
            q = q.time_in_force(time_in_force.to_string());
        }
        if let Some(expires_at) = patch.expires_at {
            q = q.expires_at(expires_at);
        }
        if let Some(broker_order_ref) = patch.broker_order_ref {
            q = q.broker_order_ref(broker_order_ref.map(str::to_string));
        }
        if let Some(notes) = patch.notes {
            q = q.notes(notes.map(str::to_string));
        }
        if let Some(status) = patch.status {
            q = q.status(status.to_string());
            // Stamp / clear the lifecycle timestamps. `expired` is passive
            // (the meaningful moment was already `expires_at`) so it
            // clears both fill + cancel.
            match status {
                "filled" => {
                    q = q.filled_at(Some(now));
                    q = q.cancelled_at(None::<jiff::Timestamp>);
                }
                "cancelled" => {
                    q = q.cancelled_at(Some(now));
                    q = q.filled_at(None::<jiff::Timestamp>);
                }
                _ => {
                    q = q.filled_at(None::<jiff::Timestamp>);
                    q = q.cancelled_at(None::<jiff::Timestamp>);
                }
            }
        }
        q.updated_at(now).exec(d).await
    })
    .await?;
    get(db, user_id, id).await
}

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let row = get(db, user_id, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
