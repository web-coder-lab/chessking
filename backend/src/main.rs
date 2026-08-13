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

use axum::{routing::get, Router};
use axum::middleware::from_fn_with_state;
use config::AppConfig;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};
use axum::http::Method;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<AppConfig>,
    pub wallet_config: Arc<wallet::config::RuntimeConfigStore>,
    pub match_registry: game::state::MatchRegistry,
    pub matchmaking: game::matchmaking::MatchmakingQueue,
    pub email: Arc<email::EmailClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::load()?;
    let pool = db::init_pool(&config.database_url).await?;

    let state = AppState {
        db: pool.clone(),
        config: Arc::new(config.clone()),
        wallet_config: Arc::new(wallet::config::RuntimeConfigStore::new()),
        match_registry: game::state::MatchRegistry::new(),
        matchmaking: game::matchmaking::MatchmakingQueue::new(),
        email: Arc::new(email::EmailClient::new(&config.smtp_host, &config.smtp_user, &config.smtp_pass, config.smtp_port)),
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

    // Doc 8 §11: "Standard rate limiting per endpoint, per user, per IP."
    // Applied globally here as the infrastructure-level backstop; a
    // tighter per-endpoint limit (e.g. login/register specifically) can
    // be layered on top of specific routes later if needed — this
    // catches the "hundreds of requests/second from one client" case
    // §11 calls out as "hard-blocked ... regardless of score."
    // SmartIpKeyExtractor reads X-Forwarded-For / X-Real-IP so rate limits
    // work correctly behind Render's reverse proxy (PeerIp alone fails with
    // "Unable To Extract Key!" on every request).
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(5)
            .burst_size(20)
            .finish()
            .expect("valid governor config")
    );
    let governor_layer = GovernorLayer { config: governor_conf };

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
    let api_v1 = Router::new()
        .merge(auth::public_routes())
        .merge(wallet::public_routes())
        .merge(game::public_routes())
        .merge(anticheat::public_routes())
        .merge(social::public_routes())
        .merge(protected)
        .layer(governor_layer);

    // CORS: allow frontend (local + Render / custom domain). FRONTEND_BASE_URL
    // is the primary origin; Any is used as fallback so free-tier previews work.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .nest("/api/v1", api_v1)
        .layer(cors)
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
