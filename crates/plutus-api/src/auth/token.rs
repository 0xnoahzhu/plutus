//! Generate and parse bearer tokens.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;

/// Generate a fresh 32-byte token, returned base64-url-encoded (no padding).
pub fn generate() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

pub fn parse_bearer(header_value: &str) -> Option<&str> {
    let trimmed = header_value.trim();
    let prefix = "Bearer ";
    if trimmed.len() <= prefix.len() {
        return None;
    }
    if !trimmed[..prefix.len()].eq_ignore_ascii_case(prefix) {
        return None;
    }
    let token = trimmed[prefix.len()..].trim();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bearer_variants() {
        assert_eq!(parse_bearer("Bearer abc"), Some("abc"));
        assert_eq!(parse_bearer("bearer xyz"), Some("xyz"));
        assert_eq!(parse_bearer("Bearer "), None);
        assert_eq!(parse_bearer("Basic abc"), None);
        assert_eq!(parse_bearer(""), None);
    }
}
