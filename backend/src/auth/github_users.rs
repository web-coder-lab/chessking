//! Durable user records in private GitHub JSON DB (Phase 5 / data plane).
//! SQLite remains ephemeral; when `GitHubStore` is configured, users survive restarts.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::db::{GitHubStore, StoreError};

use super::errors::AuthError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhUser {
    pub id: String,
    pub username: String,
    pub username_lower: String,
    pub email: String,
    pub password_hash: String,
    pub email_verified: bool,
    pub rating: i64,
    pub coin_balance: i64,
    pub two_fa_enabled: bool,
    pub two_fa_secret: Option<String>,
    pub role: String,
    pub status: String,
    pub avatar_id: Option<String>,
    pub banner_id: Option<String>,
    pub bio: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn map_err(e: StoreError) -> AuthError {
    tracing::error!("github store: {e}");
    AuthError::Internal
}

pub async fn save_user(store: &GitHubStore, user: &GhUser) -> Result<(), AuthError> {
    // Best-effort overwrite: fetch sha if exists
    let sha = match store.get_json::<Value>("users", &user.id).await {
        Ok((_, s)) => Some(s),
        Err(StoreError::NotFound) => None,
        Err(e) => return Err(map_err(e)),
    };
    store
        .put_json(
            "users",
            &user.id,
            user,
            sha.as_deref(),
            &format!("user upsert {}", user.username_lower),
        )
        .await
        .map_err(map_err)?;

    // Update email index
    upsert_index_map(store, "users_by_email", &user.email, &user.id).await?;
    upsert_index_map(store, "users_by_username", &user.username_lower, &user.id).await?;
    Ok(())
}

async fn upsert_index_map(
    store: &GitHubStore,
    index_name: &str,
    key: &str,
    user_id: &str,
) -> Result<(), AuthError> {
    let (mut map, sha) = match store.get_index::<HashMap<String, String>>(index_name).await {
        Ok(v) => v,
        Err(StoreError::NotFound) => (HashMap::new(), String::new()),
        Err(e) => return Err(map_err(e)),
    };
    map.insert(key.to_string(), user_id.to_string());
    let sha_opt = if sha.is_empty() { None } else { Some(sha.as_str()) };
    store
        .put_index(
            index_name,
            &map,
            sha_opt,
            &format!("index {index_name} set {key}"),
        )
        .await
        .map_err(map_err)?;
    Ok(())
}

pub async fn find_user_id_by_identifier(
    store: &GitHubStore,
    identifier_lower: &str,
) -> Result<Option<String>, AuthError> {
    // email index
    if let Ok((map, _)) = store
        .get_index::<HashMap<String, String>>("users_by_email")
        .await
    {
        if let Some(id) = map.get(identifier_lower) {
            return Ok(Some(id.clone()));
        }
    }
    if let Ok((map, _)) = store
        .get_index::<HashMap<String, String>>("users_by_username")
        .await
    {
        if let Some(id) = map.get(identifier_lower) {
            return Ok(Some(id.clone()));
        }
    }
    Ok(None)
}

pub async fn get_user(store: &GitHubStore, user_id: &str) -> Result<Option<GhUser>, AuthError> {
    match store.get_json::<GhUser>("users", user_id).await {
        Ok((u, _)) => Ok(Some(u)),
        Err(StoreError::NotFound) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

pub async fn username_or_email_taken(
    store: &GitHubStore,
    username_lower: &str,
    email_lower: &str,
) -> Result<(bool, bool), AuthError> {
    let mut user_taken = false;
    let mut email_taken = false;
    if let Ok((map, _)) = store
        .get_index::<HashMap<String, String>>("users_by_username")
        .await
    {
        user_taken = map.contains_key(username_lower);
    }
    if let Ok((map, _)) = store
        .get_index::<HashMap<String, String>>("users_by_email")
        .await
    {
        email_taken = map.contains_key(email_lower);
    }
    Ok((user_taken, email_taken))
}

pub fn new_user(
    id: String,
    username: String,
    email: String,
    password_hash: String,
) -> GhUser {
    let now = Utc::now().to_rfc3339();
    GhUser {
        id,
        username_lower: username.to_lowercase(),
        username,
        email,
        password_hash,
        email_verified: true, // SMTP often unset on free tier — allow login after register
        rating: 1200,
        coin_balance: 0,
        two_fa_enabled: false,
        two_fa_secret: None,
        role: "user".into(),
        status: "active".into(),
        avatar_id: None,
        banner_id: None,
        bio: None,
        created_at: now.clone(),
        updated_at: now,
    }
}
