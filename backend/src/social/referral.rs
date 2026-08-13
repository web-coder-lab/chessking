use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::wallet::ledger::{apply_ledger_entry, LedgerEntryInput};
use super::errors::SocialError;

#[derive(Debug, Serialize)]
pub struct ReferralLinkResponse { pub invite_link_code: String, pub share_url: String }

pub async fn get_referral_link(pool: &SqlitePool, user_id: &str) -> Result<ReferralLinkResponse, SocialError> {
    let row: (String,) = sqlx::query_as("SELECT referral_code FROM users WHERE id = ?")
        .bind(user_id).fetch_one(pool).await?;
    Ok(ReferralLinkResponse {
        invite_link_code: row.0.clone(),
        share_url: format!("https://chessking.app/invite/{}", row.0),
    })
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReferralProgressRow {
    pub referral_id: String, pub username: String, pub spent: i64, pub target: i64, pub claimable: i64,
}

/// Doc 9 Sec8: GET /referral/progress -> { referrals: [{ username, spent, target, claimable }] }
pub async fn referral_progress(pool: &SqlitePool, user_id: &str) -> Result<Vec<ReferralProgressRow>, SocialError> {
    let rows = sqlx::query_as::<_, ReferralProgressRow>(
        "SELECT r.id AS referral_id, u.username, r.invited_topup_pkr AS spent, r.invited_topup_target_pkr AS target,
                (r.invited_topup_pkr >= r.invited_topup_target_pkr AND r.reward_claimed = 0) AS claimable
         FROM referrals r JOIN users u ON u.id = r.invited_id
         WHERE r.inviter_id = ?
         ORDER BY r.created_at DESC"
    ).bind(user_id).fetch_all(pool).await?;
    Ok(rows)
}

#[derive(Debug, Serialize)]
pub struct ClaimReferralResponse { pub status: String, pub coins_awarded: i64 }

/// Doc 9 Sec8: POST /referral/{referral_id}/claim. Doc 5-adjacent
/// referral rule: reward only claimable once the invited user's
/// cumulative top-up reaches the target, and only once per referral.
/// Doc 8 Sec8 Referral Shield: same-device/IP correlation withholds this
/// automatically (device_fingerprint::referral_shares_device_or_ip is
/// checked at referral-creation time, not claim time, since that's when
/// the correlation actually happens).
pub async fn claim_referral(pool: &SqlitePool, user_id: &str, referral_id: &str) -> Result<ClaimReferralResponse, SocialError> {
    #[derive(sqlx::FromRow)]
    struct Row { inviter_id: String, invited_topup_pkr: i64, invited_topup_target_pkr: i64, reward_claimed: i64, reward_coins: i64 }
    let row = sqlx::query_as::<_, Row>(
        "SELECT inviter_id, invited_topup_pkr, invited_topup_target_pkr, reward_claimed, reward_coins FROM referrals WHERE id = ?"
    ).bind(referral_id).fetch_optional(pool).await?.ok_or(SocialError::NotFound)?;

    if row.inviter_id != user_id {
        return Err(SocialError::Unauthorized);
    }
    if row.reward_claimed == 1 {
        return Err(SocialError::AlreadyClaimed);
    }
    if row.invited_topup_pkr < row.invited_topup_target_pkr {
        return Err(SocialError::NotClaimable);
    }

    apply_ledger_entry(pool, LedgerEntryInput {
        user_id, log_type: "referral_reward", amount: row.reward_coins,
        reference_id: Some(referral_id), ip_address: None, device_fingerprint: None,
    }).await.map_err(|_| SocialError::Internal)?;

    sqlx::query("UPDATE referrals SET reward_claimed = 1, claimed_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339()).bind(referral_id).execute(pool).await?;

    Ok(ClaimReferralResponse { status: "claimed".to_string(), coins_awarded: row.reward_coins })
}
