use serde::Serialize;
use sqlx::SqlitePool;

use super::errors::SocialError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LeaderboardRow { pub rank: i64, pub username: String, pub rating: i64 }

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse { pub rankings: Vec<LeaderboardRow>, pub my_rank: Option<i64> }

/// Doc 9 Sec10: GET /leaderboard, query scope: global|country|province,
/// scope_value?, page, limit.
pub async fn get_leaderboard(
    pool: &SqlitePool,
    user_id: &str,
    scope: &str,
    scope_value: Option<&str>,
    page: i64,
    limit: i64,
) -> Result<LeaderboardResponse, SocialError> {
    let limit = limit.clamp(1, 100);
    let offset = (page.max(1) - 1) * limit;

    let rankings = match scope {
        "country" if scope_value.is_some() => {
            sqlx::query_as::<_, LeaderboardRow>(
                "SELECT RANK() OVER (ORDER BY rating DESC) AS rank, username, rating
                 FROM users WHERE country_code = ? ORDER BY rating DESC LIMIT ? OFFSET ?"
            ).bind(scope_value).bind(limit).bind(offset).fetch_all(pool).await?
        }
        "province" if scope_value.is_some() => {
            sqlx::query_as::<_, LeaderboardRow>(
                "SELECT RANK() OVER (ORDER BY rating DESC) AS rank, username, rating
                 FROM users WHERE province = ? ORDER BY rating DESC LIMIT ? OFFSET ?"
            ).bind(scope_value).bind(limit).bind(offset).fetch_all(pool).await?
        }
        _ => {
            sqlx::query_as::<_, LeaderboardRow>(
                "SELECT RANK() OVER (ORDER BY rating DESC) AS rank, username, rating
                 FROM users ORDER BY rating DESC LIMIT ? OFFSET ?"
            ).bind(limit).bind(offset).fetch_all(pool).await?
        }
    };

    let my_rank: Option<(i64,)> = sqlx::query_as(
        "SELECT rank FROM (SELECT id, RANK() OVER (ORDER BY rating DESC) AS rank FROM users) WHERE id = ?"
    ).bind(user_id).fetch_optional(pool).await?;

    Ok(LeaderboardResponse { rankings, my_rank: my_rank.map(|(r,)| r) })
}
