//! Append-only audit trail. `before` and `after` are JSON snapshots of the
//! affected row. `request_id` correlates entries written in the same HTTP
//! request.

#[derive(Debug, toasty::Model)]
#[table = "audit_log"]
pub struct AuditLog {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub entity_type: String,
    #[index]
    pub entity_id: String, // stringified for heterogeneity
    pub action: String,    // CREATE / UPDATE / DELETE
    pub actor_kind: String,
    pub actor_id: Option<i64>,
    pub actor_label: String,
    pub before: Option<String>, // JSON
    pub after: Option<String>,  // JSON
    pub request_id: String,
    pub created_at: jiff::Timestamp,
}
