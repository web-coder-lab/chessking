use serde::Serialize;
use sqlx::SqlitePool;

use super::errors::SocialError;

#[derive(Debug, Serialize)]
pub struct ContentResponse { pub content: String }

fn fallback_legal(key: &str) -> &'static str {
    match key {
        "privacy_policy" => "Genius Clan respects your privacy. Account data is stored securely. We do not sell personal data. Contact support for data requests.",
        "terms_of_service" => "By using Genius Clan you agree to fair play. Cheating, multi-accounting abuse, and harassment may result in suspension or ban.",
        "about" => "Genius Clan is a multiplayer chess platform: ranked and casual matches, shop cosmetics, gifts, and social features.",
        _ => "",
    }
}

pub async fn get_legal_page(pool: &SqlitePool, key: &str) -> Result<ContentResponse, SocialError> {
    let row: Option<(Option<String>,)> = sqlx::query_as("SELECT content FROM static_pages WHERE key = ?")
        .bind(key).fetch_optional(pool).await.unwrap_or(None);
    let content = row
        .and_then(|(c,)| c)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback_legal(key).to_string());
    Ok(ContentResponse { content })
}

#[derive(Debug, Serialize)]
pub struct SupportInfoResponse { pub email: String }

pub async fn get_support_info(pool: &SqlitePool) -> Result<SupportInfoResponse, SocialError> {
    let row: Option<(Option<String>,)> = sqlx::query_as("SELECT content FROM static_pages WHERE key = 'support_email'")
        .fetch_optional(pool).await.unwrap_or(None);
    Ok(SupportInfoResponse {
        email: row
            .and_then(|(c,)| c)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "support@genius-clan.app".into()),
    })
}
