//! Durable sessions in private GitHub JSON DB (Part 2).
//! Collection: data/sessions/{session_id}.json
//! Index: data/indexes/sessions_by_hash.json  (refresh_hash → session_id)

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::db::{GitHubStore, StoreError};

use super::errors::AuthError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhSession {
    pub id: String,
    pub user_id: String,
    pub refresh_token_hash: String,
    pub previous_refresh_token_hash: Option<String>,
    pub device_fingerprint: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub expires_at: String,
    pub last_seen_at: Option<String>,
}

fn map_err(e: StoreError) -> AuthError {
    tracing::error!("github sessions store: {e}");
    AuthError::Internal
}

pub async fn save_session(store: &GitHubStore, session: &GhSession) -> Result<(), AuthError> {
    let sha = match store.get_json::<Value>("sessions", &session.id).await {
        Ok((_, s)) => Some(s),
        Err(StoreError::NotFound) => None,
        Err(e) => return Err(map_err(e)),
    };
    store
        .put_json(
            "sessions",
            &session.id,
            session,
            sha.as_deref(),
            &format!("session upsert {}", session.id),
        )
        .await
        .map_err(map_err)?;

    // hash → session_id index
    let (mut map, idx_sha) = match store.get_index::<HashMap<String, String>>("sessions_by_hash").await {
        Ok(v) => v,
        Err(StoreError::NotFound) => (HashMap::new(), String::new()),
        Err(e) => return Err(map_err(e)),
    };
    // remove old hash keys pointing at this session
    map.retain(|_, sid| sid != &session.id);
    map.insert(session.refresh_token_hash.clone(), session.id.clone());
    if let Some(prev) = &session.previous_refresh_token_hash {
        // keep previous hash mapped for reuse detection briefly
        map.insert(prev.clone(), session.id.clone());
    }
    let sha_opt = if idx_sha.is_empty() { None } else { Some(idx_sha.as_str()) };
    store
        .put_index(
            "sessions_by_hash",
            &map,
            sha_opt,
            &format!("sessions_by_hash {}", session.id),
        )
        .await
        .map_err(map_err)?;
    Ok(())
}

pub async fn get_session(store: &GitHubStore, session_id: &str) -> Result<Option<GhSession>, AuthError> {
    match store.get_json::<GhSession>("sessions", session_id).await {
        Ok((s, _)) => Ok(Some(s)),
        Err(StoreError::NotFound) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

pub async fn find_by_refresh_hash(
    store: &GitHubStore,
    hash: &str,
) -> Result<Option<GhSession>, AuthError> {
    let map = match store.get_index::<HashMap<String, String>>("sessions_by_hash").await {
        Ok((m, _)) => m,
        Err(StoreError::NotFound) => return Ok(None),
        Err(e) => return Err(map_err(e)),
    };
    let Some(sid) = map.get(hash) else {
        return Ok(None);
    };
    get_session(store, sid).await
}

pub async fn deactivate(store: &GitHubStore, session_id: &str) -> Result<(), AuthError> {
    let Some(mut s) = get_session(store, session_id).await? else {
        return Ok(());
    };
    s.is_active = false;
    save_session(store, &s).await
}

pub fn new_session(
    session_id: String,
    user_id: &str,
    refresh_hash: &str,
    device_fingerprint: Option<&str>,
) -> GhSession {
    let now = Utc::now();
    GhSession {
        id: session_id,
        user_id: user_id.to_string(),
        refresh_token_hash: refresh_hash.to_string(),
        previous_refresh_token_hash: None,
        device_fingerprint: device_fingerprint.map(|s| s.to_string()),
        is_active: true,
        created_at: now.to_rfc3339(),
        expires_at: (now + Duration::days(3)).to_rfc3339(),
        last_seen_at: Some(now.to_rfc3339()),
    }
}
