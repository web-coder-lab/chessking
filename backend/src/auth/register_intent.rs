//! Part 10 — email-only signup intent (no user row until complete-signup).

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::db::{GitHubStore, StoreError};

use super::errors::AuthError;
use super::github_users;
use super::jwt::{generate_opaque_refresh_token, hash_refresh_token};
use super::validation::validate_email;

#[derive(Debug, Deserialize)]
pub struct RegisterIntentRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterIntentRecord {
    pub email: String,
    pub token_hash: String,
    pub created_at: String,
    pub expires_at: String,
    pub used: bool,
}

fn map_err(e: StoreError) -> AuthError {
    tracing::error!("register_intent store: {e}");
    AuthError::Internal
}

/// Public response always same shape (no email enumeration).
#[derive(Debug, Serialize)]
pub struct RegisterIntentResponse {
    pub status: String,
    pub email_sent: bool,
    pub message: String,
}

pub async fn create_intent(
    email_client: &crate::email::EmailClient,
    frontend_base_url: &str,
    gh: Option<&GitHubStore>,
    req: RegisterIntentRequest,
) -> Result<RegisterIntentResponse, AuthError> {
    let email_lower = req.email.trim().to_lowercase();
    validate_email(&email_lower)?;

    // If already registered on GitHub, still return generic success (anti-enumeration)
    // but skip sending a new intent if we can detect existing user.
    if let Some(store) = gh {
        if github_users::find_user_id_by_identifier(store, &email_lower)
            .await?
            .is_some()
        {
            return Ok(RegisterIntentResponse {
                status: "intent_sent".into(),
                email_sent: false,
                message: "If this email can be used, you will receive a signup link shortly.".into(),
            });
        }
    }

    let Some(store) = gh else {
        // Without durable store, intents would vanish on restart
        return Ok(RegisterIntentResponse {
            status: "intent_unavailable".into(),
            email_sent: false,
            message: "Signup by link is temporarily unavailable. Use full Register.".into(),
        });
    };

    let token_plain = generate_opaque_refresh_token();
    let token_hash = hash_refresh_token(&token_plain);
    let now = Utc::now();
    let record = RegisterIntentRecord {
        email: email_lower.clone(),
        token_hash: token_hash.clone(),
        created_at: now.to_rfc3339(),
        expires_at: (now + Duration::minutes(30)).to_rfc3339(),
        used: false,
    };

    // Store by token_hash id so complete-signup can look up without listing all
    let id = token_hash.clone();
    let sha = match store.get_json::<Value>("register_intents", &id).await {
        Ok((_, s)) => Some(s),
        Err(StoreError::NotFound) => None,
        Err(e) => return Err(map_err(e)),
    };
    store
        .put_json(
            "register_intents",
            &id,
            &record,
            sha.as_deref(),
            &format!("register intent {}", email_lower),
        )
        .await
        .map_err(map_err)?;

    // Also index email → latest hash (optional overwrite)
    let (mut map, idx_sha) = match store
        .get_index::<std::collections::HashMap<String, String>>("register_intents_by_email")
        .await
    {
        Ok(v) => v,
        Err(StoreError::NotFound) => (Default::default(), String::new()),
        Err(e) => return Err(map_err(e)),
    };
    map.insert(email_lower.clone(), id);
    let sha_opt = if idx_sha.is_empty() {
        None
    } else {
        Some(idx_sha.as_str())
    };
    let _ = store
        .put_index(
            "register_intents_by_email",
            &map,
            sha_opt,
            "register_intents_by_email",
        )
        .await;

    let email_sent = match email_client
        .send_complete_signup_email(&email_lower, &token_plain, frontend_base_url)
        .await
    {
        Ok(()) => true,
        Err(e) => {
            tracing::error!(?e, to = %email_lower, "complete-signup email failed");
            false
        }
    };

    Ok(RegisterIntentResponse {
        status: if email_sent {
            "intent_sent".into()
        } else {
            "intent_email_failed".into()
        },
        email_sent,
        message: if email_sent {
            "Check your email for a link to finish signup.".into()
        } else {
            "Could not send signup email. Try again later or use full Register.".into()
        },
    })
}

pub async fn consume_intent(
    store: &GitHubStore,
    token_plain: &str,
) -> Result<RegisterIntentRecord, AuthError> {
    let token_hash = hash_refresh_token(token_plain);
    let (mut record, sha) = match store
        .get_json::<RegisterIntentRecord>("register_intents", &token_hash)
        .await
    {
        Ok(v) => v,
        Err(StoreError::NotFound) => return Err(AuthError::Unauthorized),
        Err(e) => return Err(map_err(e)),
    };
    if record.used {
        return Err(AuthError::Unauthorized);
    }
    let exp = chrono::DateTime::parse_from_rfc3339(&record.expires_at)
        .map_err(|_| AuthError::Internal)?;
    if Utc::now() > exp {
        return Err(AuthError::Unauthorized);
    }
    record.used = true;
    store
        .put_json(
            "register_intents",
            &token_hash,
            &record,
            Some(&sha),
            "consume register intent",
        )
        .await
        .map_err(map_err)?;
    Ok(record)
}
