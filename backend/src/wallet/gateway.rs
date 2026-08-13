use async_trait::async_trait;
use serde_json::Value;

use super::errors::WalletError;

/// Doc 5 §2 steps 4b-4c: "Calls the selected gateway's API (server-to-
/// server, via Reqwest) to create a payment session/order. Returns a
/// redirect URL / payment token." Each real gateway (JazzCash, EasyPaisa,
/// Google Pay) implements this trait with its own actual API calls — the
/// exact request/response shapes are gateway-specific and documented by
/// each provider, not by this spec, so they're marked TODO here rather
/// than guessed at.
#[async_trait]
pub trait PaymentGateway: Send + Sync {
    /// Creates a payment session with the gateway. Returns (gateway's own
    /// transaction/order id, checkout redirect URL or token).
    async fn create_session(&self, amount_pkr: i64, our_transaction_id: &str) -> Result<(String, String), WalletError>;

    /// Doc 5 §3 step 1: verify the webhook signature using the gateway's
    /// documented method (HMAC, secret key, or certificate).
    fn verify_webhook_signature(&self, raw_body: &[u8], signature_header: &str, secret: &str) -> bool;

    /// Extracts the gateway's own transaction id and success/failure
    /// status from a verified webhook payload, for the §3 step 2 lookup.
    fn parse_webhook(&self, payload: &Value) -> Option<WebhookOutcome>;
}

pub struct WebhookOutcome {
    pub gateway_transaction_id: String,
    pub success: bool,
}

/// §8: gateway credentials come from RuntimeConfigStore (app_config),
/// never hardcoded — each gateway struct is constructed per-request with
/// its current key/secret/merchant_id already resolved.
pub struct JazzCashGateway {
    pub merchant_id: String,
    pub api_key: String,
    pub secret: String,
}

#[async_trait]
impl PaymentGateway for JazzCashGateway {
    async fn create_session(&self, amount_pkr: i64, our_transaction_id: &str) -> Result<(String, String), WalletError> {
        // TODO: real JazzCash Mobile Account / Card API call via reqwest,
        // per JazzCash's own integration docs (endpoint, field names, and
        // hash algorithm are provider-specific and not given in Doc 5).
        // The shape below is a structurally-correct placeholder so the
        // rest of the deposit flow (Doc 5 §2) is wired end-to-end.
        tracing::warn!("JazzCashGateway::create_session is a stub — wire real API before going live");
        let _ = (&self.merchant_id, &self.api_key, amount_pkr);
        Ok((format!("jc_{our_transaction_id}"), format!("https://sandbox.jazzcash.com.pk/pay/{our_transaction_id}")))
    }

    fn verify_webhook_signature(&self, raw_body: &[u8], signature_header: &str, secret: &str) -> bool {
        verify_hmac_sha256(raw_body, signature_header, secret)
    }

    fn parse_webhook(&self, payload: &Value) -> Option<WebhookOutcome> {
        parse_generic_webhook(payload)
    }
}

pub struct EasyPaisaGateway {
    pub merchant_id: String,
    pub api_key: String,
    pub secret: String,
}

#[async_trait]
impl PaymentGateway for EasyPaisaGateway {
    async fn create_session(&self, amount_pkr: i64, our_transaction_id: &str) -> Result<(String, String), WalletError> {
        tracing::warn!("EasyPaisaGateway::create_session is a stub — wire real API before going live");
        let _ = (&self.merchant_id, &self.api_key, amount_pkr);
        Ok((format!("ep_{our_transaction_id}"), format!("https://sandbox.easypaisa.com.pk/pay/{our_transaction_id}")))
    }

    fn verify_webhook_signature(&self, raw_body: &[u8], signature_header: &str, secret: &str) -> bool {
        verify_hmac_sha256(raw_body, signature_header, secret)
    }

    fn parse_webhook(&self, payload: &Value) -> Option<WebhookOutcome> {
        parse_generic_webhook(payload)
    }
}

pub struct GooglePayGateway {
    pub merchant_id: String,
    pub api_key: String,
    pub secret: String,
}

#[async_trait]
impl PaymentGateway for GooglePayGateway {
    async fn create_session(&self, amount_pkr: i64, our_transaction_id: &str) -> Result<(String, String), WalletError> {
        tracing::warn!("GooglePayGateway::create_session is a stub — wire real Google Pay API before going live");
        let _ = (&self.merchant_id, &self.api_key, amount_pkr);
        Ok((format!("gp_{our_transaction_id}"), format!("https://pay.google.com/gp/p/ui/pay?tx={our_transaction_id}")))
    }

    fn verify_webhook_signature(&self, raw_body: &[u8], signature_header: &str, secret: &str) -> bool {
        verify_hmac_sha256(raw_body, signature_header, secret)
    }

    fn parse_webhook(&self, payload: &Value) -> Option<WebhookOutcome> {
        parse_generic_webhook(payload)
    }
}

/// Generic HMAC-SHA256 signature check — the actual algorithm/header name
/// differs per gateway in production (check each provider's webhook docs)
/// but this is the standard pattern all three follow.
fn verify_hmac_sha256(raw_body: &[u8], signature_header: &str, secret: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else { return false };
    mac.update(raw_body);
    let expected = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison to avoid timing side-channels.
    expected.len() == signature_header.len()
        && expected.bytes().zip(signature_header.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

fn parse_generic_webhook(payload: &Value) -> Option<WebhookOutcome> {
    let gateway_transaction_id = payload.get("transaction_id")?.as_str()?.to_string();
    let status = payload.get("status")?.as_str()?;
    Some(WebhookOutcome {
        gateway_transaction_id,
        success: status.eq_ignore_ascii_case("success") || status.eq_ignore_ascii_case("paid"),
    })
}
