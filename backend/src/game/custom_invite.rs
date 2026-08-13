use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::errors::GameError;

#[derive(Debug, Deserialize)]
pub struct SendInviteRequest {
    pub receiver_username: String,
}

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub invite_id: String,
}

const INVITE_EXPIRY_HOURS: i64 = 24; // Sec6: "reasonable window, e.g. 24 hours"

/// Doc 7 Sec6 steps 3-4: creates a pending invite. Notifying the receiver
/// (real-time popup if online, notification if offline — step 6a/6b) is
/// handled by the notifications module once it exists; this function's
/// job is strictly the invite row + validation.
pub async fn send_invite(pool: &SqlitePool, sender_id: &str, req: SendInviteRequest) -> Result<InviteResponse, GameError> {
    let receiver: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username_lower = ?"
    )
    .bind(req.receiver_username.to_lowercase())
    .fetch_optional(pool)
    .await?;
    let Some((receiver_id,)) = receiver else {
        return Err(GameError::ReceiverNotFound);
    };
    if receiver_id == sender_id {
        return Err(GameError::CannotInviteSelf);
    }

    let invite_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO custom_match_invites (id, sender_id, receiver_id, status, created_at)
         VALUES (?, ?, ?, 'pending', ?)"
    )
    .bind(&invite_id)
    .bind(sender_id)
    .bind(&receiver_id)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    let sender_username: (String,) = sqlx::query_as("SELECT username FROM users WHERE id = ?")
        .bind(sender_id)
        .fetch_one(pool)
        .await?;

    // notifications::create_notification exists and is fully built (see
    // social/notifications.rs) but had no callers anywhere in the
    // codebase - this is the "once it exists" this function's own
    // earlier comment was waiting on. Fire-and-forget: a failed
    // notification insert should never fail the invite itself.
    let _ = crate::social::notifications::create_notification(
        pool,
        &receiver_id,
        "custom_match_invite",
        "New match request",
        Some(&format!("{} wants to play a custom match with you.", sender_username.0)),
        Some(&invite_id),
    ).await;

    Ok(InviteResponse { invite_id })
}

/// Doc 7 Sec6 step 7a: accept -> create the matches row (match_type =
/// "custom"), both clients routed to the board. Returns the new match_id
/// so the caller's WS/route layer can register it in MatchRegistry the
/// same way a matchmaking-queue pairing does.
pub async fn accept_invite(pool: &SqlitePool, receiver_id: &str, invite_id: &str) -> Result<(String, String, String), GameError> {
    #[derive(sqlx::FromRow)]
    struct InviteRow { sender_id: String, receiver_id: String, status: String, created_at: String }

    let invite = sqlx::query_as::<_, InviteRow>(
        "SELECT sender_id, receiver_id, status, created_at FROM custom_match_invites WHERE id = ?"
    )
    .bind(invite_id)
    .fetch_optional(pool)
    .await?
    .ok_or(GameError::InviteNotFound)?;

    if invite.receiver_id != receiver_id || invite.status != "pending" {
        return Err(GameError::InviteNotFound);
    }

    let created_at = chrono::DateTime::parse_from_rfc3339(&invite.created_at).map_err(|_| GameError::Internal)?;
    if Utc::now().signed_duration_since(created_at) > Duration::hours(INVITE_EXPIRY_HOURS) {
        sqlx::query("UPDATE custom_match_invites SET status = 'expired' WHERE id = ?")
            .bind(invite_id).execute(pool).await?;
        return Err(GameError::InviteNotFound);
    }

    // Random white/black assignment, same as Quick Match (Sec2 step 4b).
    use rand::Rng;
    let sender_is_white = rand::thread_rng().gen_bool(0.5);
    let (white_id, black_id) = if sender_is_white {
        (invite.sender_id.clone(), invite.receiver_id.clone())
    } else {
        (invite.receiver_id.clone(), invite.sender_id.clone())
    };

    let match_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO matches (id, player_white_id, player_black_id, match_type, status, white_rating_before, black_rating_before, started_at)
         SELECT ?, ?, ?, 'custom', 'in_progress', u1.rating, u2.rating, ?
         FROM users u1, users u2 WHERE u1.id = ? AND u2.id = ?"
    )
    .bind(&match_id)
    .bind(&white_id)
    .bind(&black_id)
    .bind(&now)
    .bind(&white_id)
    .bind(&black_id)
    .execute(pool)
    .await?;

    sqlx::query("UPDATE custom_match_invites SET status = 'accepted', match_id = ? WHERE id = ?")
        .bind(&match_id)
        .bind(invite_id)
        .execute(pool)
        .await?;

    Ok((match_id, white_id, black_id))
}

/// Doc 7 Sec6 step 7b: decline -> sender's Waiting screen updates.
pub async fn decline_invite(pool: &SqlitePool, receiver_id: &str, invite_id: &str) -> Result<(), GameError> {
    let result = sqlx::query(
        "UPDATE custom_match_invites SET status = 'declined' WHERE id = ? AND receiver_id = ? AND status = 'pending'"
    )
    .bind(invite_id)
    .bind(receiver_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(GameError::InviteNotFound);
    }
    Ok(())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InviteHistoryRow {
    pub id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub status: String,
    pub match_id: Option<String>,
    pub created_at: String,
}

/// Doc 7 Sec6 step 8: "Invite history (sent/received) is retained and
/// viewable." Also doubles as the sender's polling source while on the
/// Waiting screen, since there's no real-time push wired for invites yet
/// (see the commented-out notify call in send_invite above) - this at
/// least makes accept/decline observable without one.
pub async fn list_invite_history(pool: &SqlitePool, user_id: &str) -> Result<Vec<InviteHistoryRow>, GameError> {
    let rows = sqlx::query_as::<_, InviteHistoryRow>(
        "SELECT id, sender_id, receiver_id, status, match_id, created_at FROM custom_match_invites
         WHERE sender_id = ? OR receiver_id = ?
         ORDER BY created_at DESC LIMIT 50"
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
