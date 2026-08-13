use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::auth::errors::AuthError;

pub struct EmailClient {
    // None when SMTP isn't configured (expected in dev per .env's own
    // comment: "leave blank in dev — email sends will just fail silently
    // in logs"). Every send_* function still runs and logs what WOULD
    // have gone out, so the calling code never needs to know or care
    // whether real SMTP is wired up.
    //
    // Uses the async transport (AsyncSmtpTransport<Tokio1Executor>), not
    // the blocking SmtpTransport — Cargo.toml enables the
    // tokio1-rustls-tls feature specifically for this, and calling the
    // blocking client from inside axum's async handlers would tie up a
    // whole tokio worker thread for the entire SMTP round-trip (DNS +
    // TCP + TLS handshake + send), stalling other in-flight requests on
    // the same worker for however long that takes.
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: String,
}

impl EmailClient {
    /// §9: "loaded from secure config" — SMTP credentials come from
    /// AppConfig (env vars / Admin Panel), never hardcoded. Does not
    /// fail/panic on missing or invalid credentials — a chess app with
    /// no email service configured (or a typo'd one) should still start
    /// up and run; only actual email sends are affected.
    ///
    /// `smtp_port`: None picks the well-known default for the chosen
    /// mode below. Gmail specifically supports both:
    ///   - 465 (implicit TLS from the first byte — `relay()`)
    ///   - 587 (plaintext then STARTTLS upgrade — `starttls_relay()`)
    /// 587/STARTTLS is used by default here since it's the more commonly
    /// documented, more widely firewall-friendly option for Gmail App
    /// Password setups; set SMTP_PORT=465 to switch to implicit TLS if
    /// 587 is blocked on your network instead.
    pub fn new(smtp_host: &str, smtp_user: &str, smtp_pass: &str, smtp_port: Option<u16>) -> Self {
        if smtp_host.is_empty() || smtp_user.is_empty() || smtp_pass.is_empty() {
            tracing::warn!("SMTP not configured (SMTP_HOST/SMTP_USER/SMTP_PASS empty) — emails will be logged, not actually sent. Set real credentials to send email.");
            return Self { transport: None, from: "noreply@chessking.app".to_string() };
        }

        let creds = Credentials::new(smtp_user.to_string(), smtp_pass.to_string());
        let port = smtp_port.unwrap_or(587);

        let builder = if port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)
        };

        let transport = match builder {
            Ok(b) => Some(b.port(port).credentials(creds).build()),
            Err(e) => {
                tracing::error!("SMTP relay setup failed for host {smtp_host}:{port} — {e:?}. Double-check SMTP_HOST is exactly right (e.g. smtp.gmail.com) and that this network allows outbound connections on that port. Emails will be logged, not sent.");
                None
            }
        };

        tracing::info!(host = smtp_host, port, user = smtp_user, "SMTP client configured");
        Self { transport, from: smtp_user.to_string() }
    }

    /// §2.3 step 8 / §2.5: verification email. `deep_link_base` is the
    /// frontend origin, e.g. https://app.chessking.pk — token is appended
    /// as a query param and consumed by the Verify Email screen.
    pub async fn send_verification_email(&self, to: &str, token: &str, deep_link_base: &str) -> Result<(), AuthError> {
        let link = format!("{deep_link_base}/verify-email?token={token}");
        let html = shell(
            GOLD, "&#9993;&#65039;",
            "Verify your email",
            "One tap and your account is active. This link works for the next 15 minutes.",
            Some(("Verify Email", &link)),
        );
        self.send(to, "Verify your Chess King account", html).await
    }

    /// Sent right after verify-email succeeds — nothing sent this before
    /// (verification existed, but nothing welcomed the new player in).
    pub async fn send_welcome_email(&self, to: &str, username: &str, deep_link_base: &str) -> Result<(), AuthError> {
        let html = shell(
            GOLD, "&#9812;",
            &format!("Welcome to the board, {username}"),
            "Your account is verified and your starting coins are already in your wallet. Find an opponent whenever you're ready.",
            Some(("Start Playing", &format!("{deep_link_base}/dashboard"))),
        );
        self.send(to, "Welcome to Chess King", html).await
    }

    /// §6 step 3: password reset email, 15-minute link.
    pub async fn send_password_reset_email(&self, to: &str, token: &str, deep_link_base: &str) -> Result<(), AuthError> {
        let link = format!("{deep_link_base}/reset-password?token={token}");
        let html = shell(
            GOLD, "&#128273;",
            "Reset your password",
            "This link expires in 15 minutes. If you didn't request this, you can safely ignore this email &mdash; your password won't change.",
            Some(("Reset Password", &link)),
        );
        self.send(to, "Reset your Chess King password", html).await
    }

    /// Doc 2 §5 case: sign-in from a device/session the account hasn't
    /// used before. Security-relevant, so it goes out regardless of
    /// whether the sign-in was actually the account owner.
    pub async fn send_new_device_login_email(&self, to: &str, browser: &str, os: &str, approx_time: &str, deep_link_base: &str) -> Result<(), AuthError> {
        let body = format!(
            "We noticed a sign-in to your account from <strong style=\"color:#F5F5F5;\">{browser}</strong> on <strong style=\"color:#F5F5F5;\">{os}</strong>, around {approx_time}.<br/><br/>If this was you, there's nothing else to do. If it wasn't, secure your account now."
        );
        let html = shell(
            GOLD, "&#128737;&#65039;",
            "New sign-in to your account",
            &body,
            Some(("Review Active Sessions", &format!("{deep_link_base}/settings/sessions"))),
        );
        self.send(to, "New sign-in to your Chess King account", html).await
    }

    /// Doc 2 §8: confirms a 2FA state change either direction - the "on"
    /// case is reassuring, the "off" case is the more security-critical
    /// one (could mean someone else disabled your account's protection).
    pub async fn send_2fa_status_email(&self, to: &str, enabled: bool, deep_link_base: &str) -> Result<(), AuthError> {
        let html = if enabled {
            shell(
                SUCCESS_GREEN, "&#128272;",
                "Two-step verification is on",
                "Your account now asks for a 6-digit code any time it's signed into from a new device. If you didn't turn this on, contact support right away.",
                Some(("Manage Security", &format!("{deep_link_base}/settings/2fa"))),
            )
        } else {
            shell(
                DANGER_RED, "&#128275;",
                "Two-step verification is off",
                "Your account no longer asks for a second code at sign-in. If you didn't turn this off, secure your account immediately &mdash; change your password and turn two-step verification back on.",
                Some(("Secure My Account", &format!("{deep_link_base}/settings/2fa"))),
            )
        };
        let subject = if enabled { "Two-step verification turned on" } else { "Two-step verification turned off" };
        self.send(to, subject, html).await
    }

    /// Doc 4 §4 step 8: "email::send_payment_success_email" - referenced
    /// by name in wallet/webhook.rs's own comments but never existed
    /// until now.
    pub async fn send_payment_confirmation_email(&self, to: &str, amount_pkr: i64, coins_credited: i64, deep_link_base: &str) -> Result<(), AuthError> {
        let body = format!(
            "Your deposit of <strong style=\"color:#F5F5F5;\">Rs {amount_pkr}</strong> is confirmed.<br/><strong style=\"color:#D4AF37; font-size:20px;\">+{coins_credited} coins</strong><br/>added to your balance."
        );
        let html = shell(
            SUCCESS_GREEN, "&#129689;",
            "Payment received",
            &body,
            Some(("View Wallet", &format!("{deep_link_base}/wallet"))),
        );
        self.send(to, "Chess King payment confirmed", html).await
    }

    /// Doc 9 §6: "'Send test email' button to verify config works before
    /// saving live" - a small, honest test message rather than repurposing
    /// one of the real transactional templates with fake data.
    pub async fn send_test_email(&self, to: &str) -> Result<(), AuthError> {
        let html = shell(
            GOLD, "&#9989;",
            "Test email received",
            "If you're reading this, your Chess King SMTP configuration is working correctly.",
            None,
        );
        self.send(to, "Chess King — SMTP test", html).await
    }

    async fn send(&self, to: &str, subject: &str, html: String) -> Result<(), AuthError> {
        let Some(transport) = &self.transport else {
            tracing::info!(to, subject, "SMTP not configured — email logged, not actually sent");
            return Ok(());
        };

        // A display name on From: (not just a bare address) and a
        // matching Reply-To are both small, well-documented signals
        // spam filters weigh — a bare "user@gmail.com" From with no
        // name looks more like bulk/auto mail than "Chess King
        // <user@gmail.com>" does.
        let from_header = format!("Chess King <{}>", self.from);
        let email = match Message::builder()
            .from(from_header.parse().map_err(|e| { tracing::error!("invalid From address {}: {e:?}", self.from); AuthError::Internal })?)
            .reply_to(self.from.parse().map_err(|e| { tracing::error!("invalid Reply-To address {}: {e:?}", self.from); AuthError::Internal })?)
            .to(to.parse().map_err(|e| { tracing::error!("invalid recipient address {to}: {e:?}"); AuthError::Internal })?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html)
        {
            Ok(m) => m,
            Err(e) => { tracing::error!("failed to build email to {to}: {e:?}"); return Err(AuthError::Internal); }
        };

        match transport.send(email).await {
            Ok(_) => {
                tracing::info!(to, subject, "email sent");
                Ok(())
            }
            Err(e) => {
                // Logged in full detail on purpose: SMTP auth/connection
                // failures are the single most common reason "email
                // isn't arriving" during setup, and lettre's error text
                // usually says exactly why (bad credentials, wrong
                // port, TLS negotiation failure, etc.) - swallowing
                // that detail would make this much harder to debug.
                tracing::error!("smtp send to {to} failed: {e:?}");
                Err(AuthError::Internal)
            }
        }
    }
}

const GOLD: &str = "#D4AF37";
const SUCCESS_GREEN: &str = "#2ECC71";
const DANGER_RED: &str = "#E74C3C";

/// Shared branded shell every email is built from - same palette as the
/// app itself (tokens.css: #0F1115 background, #1A1D23 card, #D4AF37
/// gold). Table-based layout with inline styles throughout since email
/// clients don't reliably support <style> blocks or modern CSS. The
/// crowned-king glyph in the header is the one signature element that
/// repeats across every email type, so any Chess King email is
/// recognizable at a glance regardless of which one it is.
fn shell(accent: &str, icon: &str, headline: &str, body_html: &str, cta: Option<(&str, &str)>) -> String {
    let cta_html = match cta {
        Some((label, url)) => format!(
            "<tr><td align=\"center\" style=\"padding: 8px 0 4px;\">\
                <a href=\"{url}\" style=\"display:inline-block; background:#D4AF37; color:#0F1115; font-family: Georgia, 'Times New Roman', serif; font-weight:bold; font-size:15px; text-decoration:none; padding:14px 34px; border-radius:8px;\">{label}</a>\
            </td></tr>\
            <tr><td style=\"padding: 16px 40px 0; font-family: Arial, Helvetica, sans-serif; font-size:12px; color:#9CA3AF; word-break:break-all; text-align:center;\">Or paste this link into your browser:<br/>{url}</td></tr>"
        ),
        None => String::new(),
    };

    format!(
        "<!DOCTYPE html>\
<html><head><meta charset=\"utf-8\"/><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"/><title>Chess King</title></head>\
<body style=\"margin:0; padding:0; background:#0F1115;\">\
<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"background:#0F1115; padding: 40px 16px;\">\
<tr><td align=\"center\">\
<table role=\"presentation\" width=\"480\" cellpadding=\"0\" cellspacing=\"0\" style=\"max-width:480px; width:100%; background:#1A1D23; border-radius:12px; border-top:3px solid {accent};\">\
  <tr><td align=\"center\" style=\"padding: 28px 40px 8px;\">\
    <span style=\"font-family: Georgia, 'Times New Roman', serif; letter-spacing: 3px; font-size:13px; color:#D4AF37; text-transform:uppercase;\">&#9812; Chess King</span>\
  </td></tr>\
  <tr><td align=\"center\" style=\"padding: 14px 40px 0; font-size: 40px; line-height:1;\">{icon}</td></tr>\
  <tr><td align=\"center\" style=\"padding: 18px 40px 0; font-family: Georgia, 'Times New Roman', serif; font-size:22px; color:#F5F5F5; font-weight:bold;\">{headline}</td></tr>\
  <tr><td align=\"center\" style=\"padding: 12px 40px 24px; font-family: Arial, Helvetica, sans-serif; font-size:14px; line-height:1.6; color:#9CA3AF;\">{body_html}</td></tr>\
  {cta_html}\
  <tr><td style=\"padding: 26px 40px 28px; border-top:1px solid #2A2E37; margin-top: 20px; font-family: Arial, Helvetica, sans-serif; font-size:12px; color:#9CA3AF; text-align:center;\">\
    Chess King &middot; Need help? Reach us through Support in the app.<br/>If you didn't expect this email, you can safely ignore it.\
  </td></tr>\
</table>\
</td></tr>\
</table>\
</body></html>"
    )
}
