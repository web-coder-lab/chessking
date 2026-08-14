//! Phase 3 — application WAF: block common probe / scanner paths before handlers run.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Paths (substring) that never belong on this API/static origin.
const BLOCKED_SUBSTRINGS: &[&str] = &[
    "/.env",
    "/.git",
    "/.svn",
    "/.hg",
    "/.aws",
    "/wp-admin",
    "/wp-login",
    "/wp-content",
    "/phpmyadmin",
    "/admin.php",
    "/xmlrpc.php",
    "/vendor/phpunit",
    "/actuator",
    "/server-status",
    "/cgi-bin",
    "/etc/passwd",
    "/proc/self",
    "..%2f",
    "%2e%2e",
    "/../",
];

pub async fn block_probes(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_ascii_lowercase();
    let raw = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("").to_ascii_lowercase();

    for needle in BLOCKED_SUBSTRINGS {
        if path.contains(needle) || raw.contains(needle) {
            tracing::warn!(path = %path, "blocked probe path");
            return (StatusCode::NOT_FOUND, "not found").into_response();
        }
    }

    // Reject obviously non-API junk methods on API host (TRACE/TRACK)
    let method = req.method().as_str();
    if method == "TRACE" || method == "TRACK" {
        return (StatusCode::METHOD_NOT_ALLOWED, "method not allowed").into_response();
    }

    next.run(req).await
}
