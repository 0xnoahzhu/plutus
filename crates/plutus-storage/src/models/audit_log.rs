//! Append-only audit trail. `before` and `after` are JSON snapshots of the
//! affected row. `request_id` correlates entries written in the same HTTP
//! request.
//!
//! `user_id` is the *owning* user — distinct from `actor_id`, which can be
//! a token id when actor_kind=api_token. For admin / anonymous / system
//! actors the owning user is the `id=0` sentinel row in `users` (admin
//! auth is env-only, so admin has no real DB row; the sentinel exists
//! solely to make the FK on user_id valid for admin-authored writes).

#[derive(Debug, toasty::Model)]
#[table = "audit_log"]
pub struct AuditLog {
    #[key]
    #[auto]
    pub id: i64,
    /// Owning user. 0 for admin/anonymous/system writes (FK-valid via
    /// the `__admin` sentinel row).
    #[index]
    pub user_id: i64,
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
