use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::auth::password::{hash_password, verify_password};
use crate::auth::validation::validate_password;
use super::errors::SocialError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FullProfile {
    pub id: String, pub username: String, pub email: String, pub bio: Option<String>,
    pub avatar_id: Option<String>, pub banner_id: Option<String>,
    pub country_code: Option<String>, pub province: Option<String>, pub rating: i64, pub coin_balance: i64,
    pub two_fa_enabled: i64, pub created_at: String,
}

pub async fn get_my_profile(pool: &SqlitePool, user_id: &str) -> Result<FullProfile, SocialError> {
    sqlx::query_as::<_, FullProfile>(
        "SELECT id, username, email, bio, avatar_id, banner_id, country_code, province, rating, coin_balance, two_fa_enabled, created_at FROM users WHERE id = ?"
    ).bind(user_id).fetch_optional(pool).await?.ok_or(SocialError::NotFound)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PublicProfile {
    pub username: String, pub bio: Option<String>, pub avatar_id: Option<String>,
    pub banner_id: Option<String>, pub country_code: Option<String>, pub rating: i64, pub created_at: String,
}

/// Doc 9 Sec2: GET /profile/{username} - public view, no email/balance.
pub async fn get_public_profile(pool: &SqlitePool, username: &str) -> Result<PublicProfile, SocialError> {
    sqlx::query_as::<_, PublicProfile>(
        "SELECT username, bio, avatar_id, banner_id, country_code, rating, created_at FROM users WHERE username_lower = ?"
    ).bind(username.to_lowercase()).fetch_optional(pool).await?.ok_or(SocialError::NotFound)
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest { pub bio: Option<String> }

pub async fn update_my_profile(pool: &SqlitePool, user_id: &str, req: UpdateProfileRequest) -> Result<FullProfile, SocialError> {
    if let Some(bio) = &req.bio {
        if bio.len() > 300 {
            return Err(SocialError::ValidationFailed("Bio must be 300 characters or fewer.".to_string()));
        }
    }
    sqlx::query("UPDATE users SET bio = COALESCE(?, bio), updated_at = ? WHERE id = ?")
        .bind(&req.bio).bind(Utc::now().to_rfc3339()).bind(user_id).execute(pool).await?;
    get_my_profile(pool, user_id).await
}

#[derive(Debug, Deserialize)]
pub struct ChangeEmailRequest { pub current_password: String, pub new_email: String }

pub async fn request_email_change(pool: &SqlitePool, user_id: &str, req: ChangeEmailRequest, email_client: &crate::email::EmailClient, frontend_base_url: &str) -> Result<(), SocialError> {
    let row: (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
        .bind(user_id).fetch_one(pool).await?;
    if !verify_password(&req.current_password, &row.0) {
        return Err(SocialError::Unauthorized);
    }

    let new_email = req.new_email.trim().to_lowercase();
    crate::auth::validation::validate_email(&new_email)
        .map_err(|_| SocialError::ValidationFailed("Enter a valid email address.".to_string()))?;

    let taken: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE email = ? AND id != ?")
        .bind(&new_email).bind(user_id).fetch_optional(pool).await?;
    if taken.is_some() {
        return Err(SocialError::ValidationFailed("That email is already in use.".to_string()));
    }

    // Invalidate any earlier pending verification/change request for this
    // account, same pattern issue_verification_token uses at registration -
    // only the most recent link should ever be valid.
    sqlx::query("UPDATE email_verification_tokens SET used = 1 WHERE user_id = ? AND used = 0")
        .bind(user_id).execute(pool).await?;

    let token_plain = crate::auth::jwt::generate_opaque_refresh_token();
    let token_hash = crate::auth::jwt::hash_refresh_token(&token_plain);
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + chrono::Duration::minutes(15);

    sqlx::query(
        "INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at, used, pending_email, created_at)
         VALUES (?, ?, ?, ?, 0, ?, ?)"
    )
    .bind(&id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at.to_rfc3339())
    .bind(&new_email)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    // Sent to the NEW address, not the current one - confirms the user
    // actually controls it before anything on the account changes.
    // users.email is only ever touched once this link is clicked
    // (see verify_email's pending_email branch).
    let _ = email_client.send_verification_email(&new_email, &token_plain, frontend_base_url).await;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest { pub current_password: String, pub new_password: String }

pub async fn change_password(pool: &SqlitePool, user_id: &str, req: ChangePasswordRequest) -> Result<(), SocialError> {
    let row: (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
        .bind(user_id).fetch_one(pool).await?;
    if !verify_password(&req.current_password, &row.0) {
        return Err(SocialError::Unauthorized);
    }
    validate_password(&req.new_password).map_err(|_| SocialError::ValidationFailed("Password is too weak.".to_string()))?;
    let new_hash = hash_password(&req.new_password).map_err(|_| SocialError::Internal)?;
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(&new_hash).bind(Utc::now().to_rfc3339()).bind(user_id).execute(pool).await?;
    sqlx::query("UPDATE sessions SET is_active = 0 WHERE user_id = ? AND is_active = 1")
        .bind(user_id).execute(pool).await?;
    Ok(())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MatchHistoryRow { pub id: String, pub match_type: String, pub result: Option<String>, pub opponent_username: Option<String>, pub started_at: String }

pub async fn match_history(pool: &SqlitePool, username: &str, page: i64, limit: i64) -> Result<Vec<MatchHistoryRow>, SocialError> {
    let user: (String,) = sqlx::query_as("SELECT id FROM users WHERE username_lower = ?")
        .bind(username.to_lowercase()).fetch_optional(pool).await?.ok_or(SocialError::NotFound)?;
    let limit = limit.clamp(1, 100);
    let offset = (page.max(1) - 1) * limit;
    // Doc 3's Recent Matches / match-history UI both need the result
    // relative to THIS user ("win"/"loss"/"draw"/"void"), not the
    // absolute white_win/black_win the row stores - and the opponent's
    // username, which the raw column set never carried at all.
    let rows = sqlx::query_as::<_, MatchHistoryRow>(
        "SELECT m.id, m.match_type,
                CASE
                    WHEN m.result IS NULL THEN NULL
                    WHEN m.result IN ('draw', 'void') THEN m.result
                    WHEN m.result = 'white_win' AND m.player_white_id = ? THEN 'win'
                    WHEN m.result = 'black_win' AND m.player_black_id = ? THEN 'win'
                    ELSE 'loss'
                END AS result,
                CASE WHEN m.player_white_id = ? THEN bu.username ELSE wu.username END AS opponent_username,
                m.started_at
         FROM matches m
         LEFT JOIN users wu ON wu.id = m.player_white_id
         LEFT JOIN users bu ON bu.id = m.player_black_id
         WHERE m.player_white_id = ? OR m.player_black_id = ?
         ORDER BY m.started_at DESC LIMIT ? OFFSET ?"
    ).bind(&user.0).bind(&user.0).bind(&user.0).bind(&user.0).bind(&user.0).bind(limit).bind(offset).fetch_all(pool).await?;
    Ok(rows)
}
