//! HMAC-signed session cookies. No server-side storage required for Phase 0;
//! re-issue on next login if you want to invalidate.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub const COOKIE_NAME: &str = "plutus_session";

/// Sign a session value as `payload.signature` where both are base64-url-no-pad.
pub fn sign(secret: &[u8], payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac");
    mac.update(payload.as_bytes());
    let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload.as_bytes());
    format!("{payload_b64}.{sig}")
}

pub fn verify(secret: &[u8], cookie: &str) -> Option<String> {
    let mut parts = cookie.splitn(2, '.');
    let payload_b64 = parts.next()?;
    let sig_b64 = parts.next()?;
    let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let sig_bytes = URL_SAFE_NO_PAD.decode(sig_b64).ok()?;
    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac");
    mac.update(&payload_bytes);
    mac.verify_slice(&sig_bytes).ok()?;
    String::from_utf8(payload_bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let secret = b"a-very-secret-key-of-decent-length-12345";
        let signed = sign(secret, "logged_in");
        assert_eq!(verify(secret, &signed).as_deref(), Some("logged_in"));
        assert!(verify(b"wrong-key-wrong-key-wrong-key-wrong-key", &signed).is_none());
        assert!(verify(secret, "garbage").is_none());
    }
}
