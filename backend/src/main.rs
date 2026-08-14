mod config;
mod db;
mod auth;
mod middleware;
mod email;
mod wallet;
mod shop;
mod game;
mod anticheat;
mod admin;
mod social;

use axum::{extract::State, routing::get, Router};
use axum::middleware::{from_fn, from_fn_with_state};
use config::AppConfig;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::{header, HeaderName, HeaderValue, Method};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<AppConfig>,
    pub wallet_config: Arc<wallet::config::RuntimeConfigStore>,
    pub match_registry: game::state::MatchRegistry,
    pub matchmaking: game::matchmaking::MatchmakingQueue,
    pub email: Arc<email::EmailClient>,
    /// Durable JSON store → private GitHub repo (None if GITHUB_DATA_TOKEN unset)
    pub github_store: Option<db::SharedGitHubStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::load()?;
    let pool = db::init_pool(&config.database_url).await?;

    let github_store = db::init_github_store_from_env();
    if github_store.is_none() {
        tracing::warn!("GITHUB_DATA_TOKEN not set — durable GitHub store offline");
    }
    if !config.ip_allowlist.is_empty() {
        tracing::warn!(
            count = config.ip_allowlist.len(),
            "IP_ALLOWLIST active — API locked to listed IPs only (frontend stays public)"
        );
    }

    let state = AppState {
        db: pool.clone(),
        config: Arc::new(config.clone()),
        wallet_config: Arc::new(wallet::config::RuntimeConfigStore::new()),
        match_registry: game::state::MatchRegistry::new(),
        matchmaking: game::matchmaking::MatchmakingQueue::new(),
        email: Arc::new(email::EmailClient::new(&config.smtp_host, &config.smtp_user, &config.smtp_pass, config.smtp_port)),
        github_store,
    };

    // Doc 5 §6: background wallet audit job, every 10 minutes.
    wallet::run_periodic_audit(pool.clone()).await;

    // Doc 7 §2 step 3b: periodic ranked-queue rating-band-widening sweep.
    game::matchmaking::spawn_periodic_matching(state.matchmaking.clone());

    // Doc 8 §1.1/§17: periodic ban-escalation sweep (the ONLY path to a
    // permanent ban — 3 consecutive High-tier evaluation cycles).
    anticheat::ban_escalation::spawn_periodic_escalation(pool.clone());

    // Doc 8 §12: periodic wallet_logs hash-chain tamper check.
    anticheat::hash_integrity::spawn_periodic_integrity_check(pool.clone());

    // Phase 6: periodic security event summary in logs
    anticheat::monitor::spawn_periodic_security_summary(pool.clone());

    // Doc 8 §11: "Standard rate limiting per endpoint, per user, per IP."
    // Applied globally here as the infrastructure-level backstop; a
    // tighter per-endpoint limit (e.g. login/register specifically) can
    // be layered on top of specific routes later if needed — this
    // catches the "hundreds of requests/second from one client" case
    // §11 calls out as "hard-blocked ... regardless of score."
    // SmartIpKeyExtractor reads X-Forwarded-For / X-Real-IP so rate limits
    // work correctly behind Render's reverse proxy (PeerIp alone fails with
    // "Unable To Extract Key!" on every request).
    // Global API backstop
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(3)
            .burst_size(15)
            .finish()
            .expect("valid governor config")
    );
    let governor_layer = GovernorLayer { config: governor_conf };

    // Phase 5: stricter limit on auth public routes (login/register/forgot/reset)
    // ~1 req/s sustained, burst 5 — credential stuffing / enum resistance
    let auth_governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(1)
            .burst_size(5)
            .finish()
            .expect("valid auth governor config")
    );
    let auth_governor_layer = GovernorLayer { config: auth_governor_conf };

    // Protected routes get the require_auth middleware layer (Doc 3 §8:
    // every backend endpoint independently re-validates the JWT).
    let protected = Router::new()
        .merge(auth::protected_routes())
        .merge(wallet::protected_routes())
        .merge(shop::protected_routes())
        .merge(game::protected_routes())
        .merge(anticheat::protected_routes())
        .merge(admin::protected_routes())
        .merge(social::protected_routes())
        .layer(from_fn_with_state(state.clone(), middleware::auth_guard::require_auth));

    // Doc 9: "All routes are prefixed /api/v1." /health is deliberately
    // left outside that prefix — it's an infra/load-balancer convention,
    // not part of the application's documented API surface.
    // Rate-limit only API routes — leave /health unrestricted so Render
    // load-balancer health checks never get 429'd.
    let auth_public = auth::public_routes().layer(auth_governor_layer);
    let api_v1 = Router::new()
        .merge(auth_public)
        .merge(wallet::public_routes())
        .merge(game::public_routes())
        .merge(anticheat::public_routes())
        .merge(social::public_routes())
        .merge(protected)
        .layer(governor_layer);

    // Phase 4 firewalls: CORS locked to known frontends (not Reflect Any).
    let frontend = state.config.frontend_base_url.trim_end_matches('/').to_string();
    let mut origins = vec![
        frontend.clone(),
        "http://localhost:5173".into(),
        "http://127.0.0.1:5173".into(),
        "https://genius-clan.onrender.com".into(),
    ];
    origins.sort();
    origins.dedup();
    let origin_list: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origin_list))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allow_credentials(false);

    // Security response headers (application firewall layer)
    let security_headers = (
        SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
        SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ),
        SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("0"),
        ),
        SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ),
        SetResponseHeaderLayer::overriding(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ),
    );

    let app = Router::new()
        .route("/health", get(health))
        .route("/health/store", get(health_store))
        .nest("/api/v1", api_v1)
        .layer(from_fn(middleware::probe_guard::block_probes))
        .layer(from_fn_with_state(state.clone(), middleware::ip_allowlist::enforce_ip_allowlist))
        .layer(cors)
        .layer(security_headers.0)
        .layer(security_headers.1)
        .layer(security_headers.2)
        .layer(security_headers.3)
        .layer(security_headers.4)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

/// Durable-store probe: confirms private GitHub repo is reachable.
async fn health_store(State(state): State<AppState>) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    match &state.github_store {
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "ok": false,
                "store": "github",
                "error": "GITHUB_DATA_TOKEN not configured"
            })),
        )
            .into_response(),
        Some(store) => match store.get_index::<serde_json::Value>("users_by_email").await {
            Ok(_) => (
                StatusCode::OK,
                axum::Json(serde_json::json!({
                    "ok": true,
                    "store": "github",
                    "repo": "genius-clan-database"
                })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(serde_json::json!({
                    "ok": false,
                    "store": "github",
                    "error": e.to_string()
                })),
            )
                .into_response(),
        },
    }
}
