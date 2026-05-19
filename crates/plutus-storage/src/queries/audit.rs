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
        plutus_core::audit::ActorKind::Anonymous => "anonymous",
        plutus_core::audit::ActorKind::System => "system",
    };
    let entity_type = entry.entity_type.to_string();
    let entity_id = entry.entity_id;
    let action = action_str.to_string();
    let actor_kind = actor_kind.to_string();
    let actor_id = entry.actor.token_id;
    let actor_label = entry.actor.label.clone();
    let before = entry.before;
    let after = entry.after;
    let request_id = entry.request_id.to_string();
    let now = jiff::Timestamp::now();
    let row = db
        .with(async |d| {
            toasty::create!(AuditLog {
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

pub async fn list(db: &Db) -> Result<Vec<AuditLog>> {
    db.with(async |d| AuditLog::all().exec(d).await)
        .await
        .map_err(Into::into)
}
