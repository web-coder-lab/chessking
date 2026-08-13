use serde::Serialize;
use sqlx::SqlitePool;

use super::errors::SocialError;

#[derive(Debug, Serialize)]
pub struct ContentResponse { pub content: String }

pub async fn get_legal_page(pool: &SqlitePool, key: &str) -> Result<ContentResponse, SocialError> {
    let row: Option<(Option<String>,)> = sqlx::query_as("SELECT content FROM static_pages WHERE key = ?")
        .bind(key).fetch_optional(pool).await?;
    Ok(ContentResponse { content: row.and_then(|(c,)| c).unwrap_or_default() })
}

#[derive(Debug, Serialize)]
pub struct SupportInfoResponse { pub email: String }

pub async fn get_support_info(pool: &SqlitePool) -> Result<SupportInfoResponse, SocialError> {
    let row: Option<(Option<String>,)> = sqlx::query_as("SELECT content FROM static_pages WHERE key = 'support_email'")
        .fetch_optional(pool).await?;
    Ok(SupportInfoResponse { email: row.and_then(|(c,)| c).unwrap_or_default() })
}
