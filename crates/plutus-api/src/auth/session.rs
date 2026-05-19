//! Session cookie. Single-user local deployment — no signing, no
//! server-side store. A static marker value paired with HttpOnly +
//! SameSite=Lax is enough to gate the UI behind a password.

pub const COOKIE_NAME: &str = "plutus_session";

/// Value we set after a successful login. Any other value (or absent
/// cookie) is treated as unauthenticated.
pub const VALUE: &str = "ok";
