//! Web session cookie. The cookie value is a random opaque id; the actual
//! identity (user_id / is_admin / username) lives in the `web_sessions` table.
//! That gives us server-side revocation: deleting the row invalidates the
//! cookie immediately.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;

pub const COOKIE_NAME: &str = "plutus_session";

/// Generate a fresh 32-byte session id, returned base64-url-encoded (no padding).
pub fn generate_id() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}
