use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use super::errors::AuthError;

/// §9 Security Hard Rules: Argon2id password hashing, unique salt per
/// password. Never store or log the plaintext password anywhere.
pub fn hash_password(plain: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| {
            tracing::error!("password hash failed: {e:?}");
            AuthError::Internal
        })
}

/// Verifies a plaintext password against a stored Argon2id hash.
/// Constant-time comparison is handled internally by the argon2 crate.
pub fn verify_password(plain: &str, stored_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok()
}
