//! Idempotency key newtype. Clients put a fresh random string in the
//! `Idempotency-Key` header; we hash + store the first response and replay it
//! on retries within the TTL window.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Accept any non-empty string up to 255 chars. Stripe limits to 255; we
    /// match that.
    pub fn new(s: impl Into<String>) -> Option<Self> {
        let s = s.into();
        if s.is_empty() || s.len() > 255 {
            None
        } else {
            Some(Self(s))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
