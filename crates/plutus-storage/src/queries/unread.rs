//! Per-user unread state for event-like entities (news, recommendations,
//! portfolio reviews, …). Reading a detail page side-effects a row into
//! `user_reads`; absence of the row means unread.
//!
//! Per-row `read_at` columns on each entity table were rejected because
//! three of the ten entities (news_items, macro_events, earnings_events)
//! are shared across users and don't carry user_id. A central join table
//! handles both cases uniformly.

use std::collections::HashMap;

use crate::db::{Db, DbError, Result};

/// Canonical set of entities that participate in unread tracking. The
/// string form is what goes into `user_reads.entity_type`; keep it stable
/// across deploys because rows already in the table won't be migrated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    News,
    MarketBrief,
    MacroEvent,
    EarningsEvent,
    Catalyst,
    ScreenerRun,
    Recommendation,
    PortfolioReview,
    CorrelationRun,
    SelfExam,
}

impl EntityKind {
    pub const ALL: &'static [EntityKind] = &[
        EntityKind::News,
        EntityKind::MarketBrief,
        EntityKind::MacroEvent,
        EntityKind::EarningsEvent,
        EntityKind::Catalyst,
        EntityKind::ScreenerRun,
        EntityKind::Recommendation,
        EntityKind::PortfolioReview,
        EntityKind::CorrelationRun,
        EntityKind::SelfExam,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            EntityKind::News => "news",
            EntityKind::MarketBrief => "market_brief",
            EntityKind::MacroEvent => "macro_event",
            EntityKind::EarningsEvent => "earnings_event",
            EntityKind::Catalyst => "catalyst",
            EntityKind::ScreenerRun => "screener_run",
            EntityKind::Recommendation => "recommendation",
            EntityKind::PortfolioReview => "portfolio_review",
            EntityKind::CorrelationRun => "correlation_run",
            EntityKind::SelfExam => "self_exam",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.as_str() == s)
    }

    /// Underlying table name in `plutus` schema.
    fn table(&self) -> &'static str {
        match self {
            EntityKind::News => "news_items",
            EntityKind::MarketBrief => "market_briefs",
            EntityKind::MacroEvent => "macro_events",
            EntityKind::EarningsEvent => "earnings_events",
            EntityKind::Catalyst => "catalysts",
            EntityKind::ScreenerRun => "screener_runs",
            EntityKind::Recommendation => "recommendations",
            EntityKind::PortfolioReview => "portfolio_reviews",
            EntityKind::CorrelationRun => "correlation_runs",
            EntityKind::SelfExam => "self_exams",
        }
    }

    /// True if the table has a `user_id` column for per-user scoping. News,
    /// macro events, and earnings events are global feeds.
    fn has_user_id(&self) -> bool {
        !matches!(
            self,
            EntityKind::News | EntityKind::MacroEvent | EntityKind::EarningsEvent
        )
    }
}

/// Idempotent mark-read. Called as a side effect from each entity's detail
/// GET handler. Re-reading is a no-op.
pub async fn mark_read(
    db: &Db,
    user_id: i64,
    kind: EntityKind,
    entity_id: i64,
) -> Result<()> {
    if user_id == 0 {
        return Ok(()); // admin / orphan sentinel has no unread state
    }
    let client = db.raw_client().await?;
    client
        .execute(
            r#"
                INSERT INTO user_reads (user_id, entity_type, entity_id, read_at)
                VALUES ($1, $2, $3, now())
                ON CONFLICT (user_id, entity_type, entity_id) DO NOTHING
            "#,
            &[&user_id, &kind.as_str(), &entity_id],
        )
        .await
        .map_err(DbError::from)?;
    Ok(())
}

/// Mark every visible entity of this kind as read for the user. Used by
/// the "标记全部已读" button on each list page. Idempotent — already-read
/// items are skipped via ON CONFLICT DO NOTHING. Returns the number of
/// fresh inserts so the UI can show a confirmation (or skip the flash
/// when nothing changed).
pub async fn mark_all_read(
    db: &Db,
    user_id: i64,
    kind: EntityKind,
) -> Result<u64> {
    if user_id == 0 {
        return Ok(0);
    }
    let client = db.raw_client().await?;
    let table = kind.table();
    let user_filter = if kind.has_user_id() {
        "WHERE t.user_id = $1"
    } else {
        ""
    };
    let sql = format!(
        r#"
            INSERT INTO user_reads (user_id, entity_type, entity_id, read_at)
            SELECT $1, $2, t.id, now()
            FROM {table} t
            {user_filter}
            ON CONFLICT (user_id, entity_type, entity_id) DO NOTHING
        "#,
    );
    let n = client
        .execute(&sql, &[&user_id, &kind.as_str()])
        .await
        .map_err(DbError::from)?;
    Ok(n)
}

/// Mark an item back as unread. Used by the optional "unread" toggle in the UI.
pub async fn unmark_read(
    db: &Db,
    user_id: i64,
    kind: EntityKind,
    entity_id: i64,
) -> Result<()> {
    if user_id == 0 {
        return Ok(());
    }
    let client = db.raw_client().await?;
    client
        .execute(
            "DELETE FROM user_reads WHERE user_id = $1 AND entity_type = $2 AND entity_id = $3",
            &[&user_id, &kind.as_str(), &entity_id],
        )
        .await
        .map_err(DbError::from)?;
    Ok(())
}

/// Unread counts per entity kind for the sidebar badge. Items created before
/// the user signed up are not counted — new users start with a clean slate.
pub async fn counts(db: &Db, user_id: i64) -> Result<HashMap<EntityKind, i64>> {
    let mut result: HashMap<EntityKind, i64> = HashMap::new();
    if user_id == 0 {
        for k in EntityKind::ALL {
            result.insert(*k, 0);
        }
        return Ok(result);
    }

    let client = db.raw_client().await?;
    let user_row = client
        .query_opt("SELECT created_at FROM users WHERE id = $1", &[&user_id])
        .await
        .map_err(DbError::from)?;
    let Some(row) = user_row else {
        for k in EntityKind::ALL {
            result.insert(*k, 0);
        }
        return Ok(result);
    };
    let watermark: jiff::Timestamp = row.get(0);

    for kind in EntityKind::ALL {
        let table = kind.table();
        let user_filter = if kind.has_user_id() {
            "t.user_id = $1 AND "
        } else {
            ""
        };
        let sql = format!(
            r#"
                SELECT COUNT(*)::BIGINT
                FROM {table} t
                WHERE {user_filter}t.created_at > $2
                  AND NOT EXISTS (
                      SELECT 1 FROM user_reads ur
                      WHERE ur.user_id = $1
                        AND ur.entity_type = $3
                        AND ur.entity_id = t.id
                  )
            "#,
        );
        let row = client
            .query_one(&sql, &[&user_id, &watermark, &kind.as_str()])
            .await
            .map_err(DbError::from)?;
        let count: i64 = row.get(0);
        result.insert(*kind, count);
    }
    Ok(result)
}

/// Fetch the `read_at` timestamps for a batch of entity ids of a single
/// kind. Returns a map keyed by entity_id; missing entries are unread.
/// Used by list handlers to surface per-row read state.
pub async fn read_ats(
    db: &Db,
    user_id: i64,
    kind: EntityKind,
    ids: &[i64],
) -> Result<HashMap<i64, jiff::Timestamp>> {
    let mut result = HashMap::new();
    if user_id == 0 || ids.is_empty() {
        return Ok(result);
    }
    let client = db.raw_client().await?;
    let rows = client
        .query(
            r#"
                SELECT entity_id, read_at
                FROM user_reads
                WHERE user_id = $1
                  AND entity_type = $2
                  AND entity_id = ANY($3)
            "#,
            &[&user_id, &kind.as_str(), &ids],
        )
        .await
        .map_err(DbError::from)?;
    for row in rows {
        let id: i64 = row.get("entity_id");
        let ts: jiff::Timestamp = row.get("read_at");
        result.insert(id, ts);
    }
    Ok(result)
}
