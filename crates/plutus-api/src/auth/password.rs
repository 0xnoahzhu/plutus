//! Argon2id password hashing. Used for regular users (rows in the `users`
//! table). The admin account does NOT go through this — admin credentials
//! live in env vars as plaintext.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a plaintext password with Argon2id + a fresh random salt.
pub fn hash(plain: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(plain.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a plaintext password against a previously stored hash. Returns
/// `false` for both wrong password and malformed hash; callers don't need to
/// distinguish.
#[must_use]
pub fn verify(plain: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_verify_roundtrip() {
        let h = hash("hunter2").expect("hash");
        assert!(verify("hunter2", &h));
        assert!(!verify("wrong", &h));
    }

    #[test]
    fn verify_rejects_malformed() {
        assert!(!verify("anything", "not-a-real-hash"));
    }
}
