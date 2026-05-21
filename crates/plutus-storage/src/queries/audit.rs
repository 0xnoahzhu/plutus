use plutus_core::audit::{Actor, AuditAction};

use crate::db::{Db, Result};
use crate::models::AuditLog;

pub struct RecordAudit<'a> {
    pub entity_type: &'a str,
    pub entity_id: String,
    pub action: AuditAction,
    pub actor: &'a Actor,
    pub before: Option<String>,
    pub after: Option<String>,
    pub request_id: &'a str,
}

pub async fn record(db: &Db, entry: RecordAudit<'_>) -> Result<AuditLog> {
    let action_str = match entry.action {
        AuditAction::Create => "CREATE",
        AuditAction::Update => "UPDATE",
        AuditAction::Delete => "DELETE",
    };
    let actor_kind = match entry.actor.kind {
        plutus_core::audit::ActorKind::Web => "web",
        plutus_core::audit::ActorKind::ApiToken => "api_token",
        plutus_core::audit::ActorKind::Admin => "admin",
        plutus_core::audit::ActorKind::Anonymous => "anonymous",
        plutus_core::audit::ActorKind::System => "system",
    };
    let entity_type = entry.entity_type.to_string();
    let entity_id = entry.entity_id;
    let action = action_str.to_string();
    let actor_kind = actor_kind.to_string();
    // Prefer the user_id (real owner of the change) over the token_id; fall
    // back to token_id so api-token actors that aren't yet user-scoped still
    // get a non-null actor_id for the audit row.
    let actor_id = entry.actor.user_id.or(entry.actor.token_id);
    // Owning user goes into its own column (FK -> users.id). Admin /
    // anonymous / system actors fall back to the `id=0` sentinel row so
    // the FK is always satisfied without inventing dummy DB rows for
    // each env-based admin login.
    let user_id = entry.actor.user_id.unwrap_or(0);
    let actor_label = entry.actor.label.clone();
    let before = entry.before;
    let after = entry.after;
    let request_id = entry.request_id.to_string();
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(AuditLog {
                user_id: user_id,
                entity_type: entity_type,
                entity_id: entity_id,
                action: action,
                actor_kind: actor_kind,
                actor_id: actor_id,
                actor_label: actor_label,
                before: before,
                after: after,
                request_id: request_id,
                created_at: now,
            })
            .exec(d)
            .await
        })
        .await?;
    Ok(row)
}

/// Newest-first. Cap at 200 rows; the home Recent Activity card slices
/// the first ~8 and the dedicated `/audit` page rarely needs more —
/// this keeps a single call lightweight without paginating yet.
const LIST_CAP: i64 = 200;

pub async fn list(db: &Db) -> Result<Vec<AuditLog>> {
    let client = db.raw_client().await?;
    let rows = client
        .query(
            "SELECT id, user_id, entity_type, entity_id, action, actor_kind, \
                    actor_id, actor_label, before, after, request_id, created_at \
               FROM audit_log \
              ORDER BY created_at DESC, id DESC \
              LIMIT $1",
            &[&LIST_CAP],
        )
        .await
        .map_err(crate::db::DbError::from)?;
    Ok(rows
        .into_iter()
        .map(|r| AuditLog {
            id: r.get("id"),
            user_id: r.get("user_id"),
            entity_type: r.get("entity_type"),
            entity_id: r.get("entity_id"),
            action: r.get("action"),
            actor_kind: r.get("actor_kind"),
            actor_id: r.get("actor_id"),
            actor_label: r.get("actor_label"),
            before: r.get("before"),
            after: r.get("after"),
            request_id: r.get("request_id"),
            created_at: r.get("created_at"),
        })
        .collect())
}
