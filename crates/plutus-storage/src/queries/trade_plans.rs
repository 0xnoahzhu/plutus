//! Trade plan + per-level CRUD. Plans are per-user, per-stock; levels
//! belong to a plan via FK (CASCADE on plan delete). Both layers run
//! through toasty — no JSONB on these tables, so the schema stays
//! straightforward.

use rust_decimal::Decimal;

use crate::db::{Db, DbError, Result};
use crate::models::{TradePlan, TradePlanLevel};

// ── Plans ────────────────────────────────────────────────────────────────

pub struct ListFilter<'a> {
    pub user_id: i64,
    pub stock_id: Option<i64>,
    pub status: Option<&'a str>,
}

pub async fn list(db: &Db, filter: ListFilter<'_>) -> Result<Vec<TradePlan>> {
    let user_id = filter.user_id;
    let status_filter = filter.status.map(str::to_string);
    let rows = db
        .with(async |d| TradePlan::all().exec(d).await)
        .await?;
    Ok(rows
        .into_iter()
        .filter(|r| r.user_id == user_id)
        .filter(|r| filter.stock_id.map_or(true, |s| r.stock_id == s))
        .filter(|r| status_filter.as_deref().map_or(true, |s| r.status == s))
        .collect())
}

pub async fn get(db: &Db, user_id: i64, id: i64) -> Result<TradePlan> {
    let row = db
        .with(async |d| TradePlan::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewPlan<'a> {
    pub user_id: i64,
    pub stock_id: i64,
    pub rationale: Option<&'a str>,
}

pub async fn create(db: &Db, input: NewPlan<'_>) -> Result<TradePlan> {
    let now = jiff::Timestamp::now();
    let user_id = input.user_id;
    let stock_id = input.stock_id;
    let rationale = input.rationale.map(str::to_string);
    let row = db
        .with(async |d| {
            toasty::create!(TradePlan {
                user_id: user_id,
                stock_id: stock_id,
                rationale: rationale,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub struct PlanPatch<'a> {
    pub rationale: Option<Option<&'a str>>,
    pub status: Option<&'a str>,
}

pub async fn update(db: &Db, user_id: i64, id: i64, patch: PlanPatch<'_>) -> Result<TradePlan> {
    let mut row = get(db, user_id, id).await?;
    let now = jiff::Timestamp::now();
    db.with(async |d| {
        let mut q = row.update();
        if let Some(rationale) = patch.rationale {
            q = q.rationale(rationale.map(str::to_string));
        }
        if let Some(status) = patch.status {
            q = q.status(status.to_string());
        }
        q.updated_at(now).exec(d).await
    })
    .await?;
    get(db, user_id, id).await
}

pub async fn delete(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let row = get(db, user_id, id).await?;
    // Levels go with the plan — the FK has ON DELETE CASCADE so we don't
    // need to walk them manually.
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}

// ── Levels ───────────────────────────────────────────────────────────────

pub async fn list_levels_for_plan(
    db: &Db,
    user_id: i64,
    plan_id: i64,
) -> Result<Vec<TradePlanLevel>> {
    // Guard via the plan first so a cross-tenant plan_id returns NotFound
    // before we leak any level rows.
    let _plan = get(db, user_id, plan_id).await?;
    let rows = db
        .with(async |d| {
            TradePlanLevel::all()
                .filter(TradePlanLevel::fields().plan_id().eq(plan_id))
                .exec(d)
                .await
        })
        .await?;
    let mut rows: Vec<TradePlanLevel> = rows.into_iter().filter(|r| r.user_id == user_id).collect();
    // sort_order asc (NULLs treated as +∞), then price asc.
    rows.sort_by(|a, b| match (a.sort_order, b.sort_order) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.price.cmp(&b.price),
    });
    Ok(rows)
}

pub async fn list_levels_for_user(
    db: &Db,
    user_id: i64,
    stock_id: Option<i64>,
    status: Option<&str>,
) -> Result<Vec<TradePlanLevel>> {
    let rows = db
        .with(async |d| TradePlanLevel::all().exec(d).await)
        .await?;
    let status_owned = status.map(str::to_string);
    Ok(rows
        .into_iter()
        .filter(|r| r.user_id == user_id)
        .filter(|r| stock_id.map_or(true, |s| r.stock_id == s))
        .filter(|r| status_owned.as_deref().map_or(true, |s| r.status == s))
        .collect())
}

pub async fn get_level(db: &Db, user_id: i64, id: i64) -> Result<TradePlanLevel> {
    let row = db
        .with(async |d| TradePlanLevel::filter_by_id(id).first().exec(d).await)
        .await?;
    match row {
        Some(r) if r.user_id == user_id => Ok(r),
        _ => Err(DbError::NotFound),
    }
}

pub struct NewLevel<'a> {
    pub plan_id: i64,
    pub kind: &'a str,
    pub price: Decimal,
    pub quantity: Option<Decimal>,
    pub fraction_pct: Option<Decimal>,
    pub notes: Option<&'a str>,
    pub sort_order: Option<i32>,
}

pub async fn add_level(db: &Db, user_id: i64, input: NewLevel<'_>) -> Result<TradePlanLevel> {
    // Resolve the parent plan first so the level can inherit stock_id +
    // user_id without trusting the caller.
    let plan = get(db, user_id, input.plan_id).await?;
    let now = jiff::Timestamp::now();
    let plan_id = plan.id;
    let stock_id = plan.stock_id;
    let kind = input.kind.to_string();
    let price = input.price;
    let quantity = input.quantity;
    let fraction_pct = input.fraction_pct;
    let notes = input.notes.map(str::to_string);
    let sort_order = input.sort_order;
    let row = db
        .with(async |d| {
            toasty::create!(TradePlanLevel {
                user_id: user_id,
                stock_id: stock_id,
                plan_id: plan_id,
                kind: kind,
                price: price,
                quantity: quantity,
                fraction_pct: fraction_pct,
                status: "active".to_string(),
                triggered_at: None::<jiff::Timestamp>,
                notes: notes,
                sort_order: sort_order,
                created_at: now,
                updated_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

pub struct LevelPatch<'a> {
    pub kind: Option<&'a str>,
    pub price: Option<Decimal>,
    pub quantity: Option<Option<Decimal>>,
    pub fraction_pct: Option<Option<Decimal>>,
    pub notes: Option<Option<&'a str>>,
    pub sort_order: Option<Option<i32>>,
    /// `active` / `triggered` / `cancelled`. Flipping to `triggered`
    /// stamps `triggered_at = now()`; the other transitions clear it.
    pub status: Option<&'a str>,
}

pub async fn update_level(
    db: &Db,
    user_id: i64,
    id: i64,
    patch: LevelPatch<'_>,
) -> Result<TradePlanLevel> {
    let mut row = get_level(db, user_id, id).await?;
    let now = jiff::Timestamp::now();
    db.with(async |d| {
        let mut q = row.update();
        if let Some(kind) = patch.kind {
            q = q.kind(kind.to_string());
        }
        if let Some(price) = patch.price {
            q = q.price(price);
        }
        if let Some(quantity) = patch.quantity {
            q = q.quantity(quantity);
        }
        if let Some(fraction_pct) = patch.fraction_pct {
            q = q.fraction_pct(fraction_pct);
        }
        if let Some(notes) = patch.notes {
            q = q.notes(notes.map(str::to_string));
        }
        if let Some(sort_order) = patch.sort_order {
            q = q.sort_order(sort_order);
        }
        if let Some(status) = patch.status {
            q = q.status(status.to_string());
            // Stamp/clear the trigger timestamp so the UI doesn't need
            // extra round-trips to keep them in sync.
            q = q.triggered_at(if status == "triggered" { Some(now) } else { None });
        }
        q.updated_at(now).exec(d).await
    })
    .await?;
    get_level(db, user_id, id).await
}

pub async fn delete_level(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let row = get_level(db, user_id, id).await?;
    db.with(async |d| row.delete().exec(d).await).await?;
    Ok(())
}
