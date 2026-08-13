use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::errors::AuthError;

/// §7: "Access token: JWT, 5-minute expiry, contains user_id, role, session_id"
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessClaims {
    pub sub: String,       // user_id
    pub role: String,
    pub session_id: String,
    pub exp: i64,
    pub iat: i64,
}

const ACCESS_TOKEN_TTL_SECS: i64 = 5 * 60; // exactly 5 minutes per spec

pub fn issue_access_token(user_id: &str, role: &str, session_id: &str, jwt_secret: &str) -> Result<String, AuthError> {
    let now = chrono::Utc::now().timestamp();
    let claims = AccessClaims {
        sub: user_id.to_string(),
        role: role.to_string(),
        session_id: session_id.to_string(),
        iat: now,
        exp: now + ACCESS_TOKEN_TTL_SECS,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes()))
        .map_err(|e| {
            tracing::error!("jwt encode failed: {e:?}");
            AuthError::Internal
        })
}

/// §8: every backend endpoint independently re-validates the JWT via
/// middleware — this is the single source of truth for "is this token good".
pub fn verify_access_token(token: &str, jwt_secret: &str) -> Result<AccessClaims, AuthError> {
    decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::Unauthorized)
}

/// §7: "Refresh token: opaque random token, 3-day expiry, stored hashed."
/// Opaque = NOT a JWT, just cryptographically random bytes. The plaintext
/// is only ever sent to the client once, never stored server-side.
pub fn generate_opaque_refresh_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Refresh tokens are hashed at rest (sha256 is fine here — it's a random
/// 256-bit token, not a low-entropy password, so no need for Argon2).
pub fn hash_refresh_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
