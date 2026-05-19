use axum::extract::State;
use axum::Json;
use serde::Serialize;

use plutus_storage::models::AuditLog;

use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AuditEntryOut {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub actor_kind: String,
    pub actor_id: Option<i64>,
    pub actor_label: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub request_id: String,
    pub created_at: String,
}

impl From<AuditLog> for AuditEntryOut {
    fn from(a: AuditLog) -> Self {
        Self {
            id: a.id,
            entity_type: a.entity_type,
            entity_id: a.entity_id,
            action: a.action,
            actor_kind: a.actor_kind,
            actor_id: a.actor_id,
            actor_label: a.actor_label,
            before: a.before,
            after: a.after,
            request_id: a.request_id,
            created_at: a.created_at.to_string(),
        }
    }
}

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<AuditEntryOut>>> {
    let rows = plutus_storage::queries::audit::list(&state.db).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
