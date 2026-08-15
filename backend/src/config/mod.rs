use std::env;

/// Server-side only configuration. Loaded once at startup from environment
/// variables (.env in dev, real env vars in production). Never serialized
/// or exposed to the frontend/browser under any circumstance.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_access_ttl_minutes: i64,
    pub jwt_refresh_ttl_days: i64,
    pub smtp_host: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_port: Option<u16>,
    pub frontend_base_url: String,
    pub jazzcash_api_key: Option<String>,
    pub easypaisa_api_key: Option<String>,
    pub googlepay_api_key: Option<String>,
    pub port: u16,
    /// Comma-separated IPs or CIDRs. Empty = public API. Non-empty = only these IPs can call API.
    pub ip_allowlist: Vec<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            // Empty or sqlite::memory: = no durable files on Render.
            // Real data: GITHUB_DATA_* → private repo genius-clan-database.
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite::memory:".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set — never use a default in production"),
            jwt_access_ttl_minutes: env::var("JWT_ACCESS_TTL_MIN")
                .unwrap_or_else(|_| "15".to_string())
                .parse()?,
            jwt_refresh_ttl_days: env::var("JWT_REFRESH_TTL_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            smtp_host: env::var("SMTP_HOST").unwrap_or_default(),
            smtp_user: env::var("SMTP_USER").unwrap_or_default(),
            smtp_pass: env::var("SMTP_PASS").unwrap_or_default().chars().filter(|c| !c.is_whitespace()).collect(),
            smtp_port: env::var("SMTP_PORT").ok().and_then(|s| s.parse().ok()),
            frontend_base_url: env::var("FRONTEND_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            jazzcash_api_key: env::var("JAZZCASH_API_KEY").ok(),
            easypaisa_api_key: env::var("EASYPAISA_API_KEY").ok(),
            googlepay_api_key: env::var("GOOGLEPAY_API_KEY").ok(),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            ip_allowlist: env::var("IP_ALLOWLIST")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        })
    }
}
