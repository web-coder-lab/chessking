//! Optional IP allowlist. When non-empty, only listed IPs may use the API
//! (except /health). Everyone else gets a branded Genius 404 — frontend
//! static site remains publicly reachable on its own host.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::IpAddr;

use crate::AppState;

fn client_ip(req: &Request<Body>) -> Option<IpAddr> {
    // Prefer first X-Forwarded-For hop (Render / Cloudflare)
    if let Some(xff) = req.headers().get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = xff.split(',').next() {
            if let Ok(ip) = first.trim().parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }
    if let Some(real) = req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()) {
        if let Ok(ip) = real.trim().parse::<IpAddr>() {
            return Some(ip);
        }
    }
    None
}

fn ip_allowed(ip: &IpAddr, allowlist: &[String]) -> bool {
    let s = ip.to_string();
    allowlist.iter().any(|entry| {
        let e = entry.trim();
        if e.is_empty() {
            return false;
        }
        // Exact IP match
        if e == s {
            return true;
        }
        // Simple prefix / CIDR-lite: "203.0.113." matches that range start
        // Full CIDR parsing kept minimal for free-tier ops
        if e.contains('/') {
            // parse "a.b.c.d/n" roughly for IPv4
            if let Some((base, bits)) = e.split_once('/') {
                if let (Ok(base_ip), Ok(prefix)) = (base.parse::<IpAddr>(), bits.parse::<u8>()) {
                    return ip_in_cidr(ip, &base_ip, prefix);
                }
            }
        }
        false
    })
}

fn ip_in_cidr(ip: &IpAddr, base: &IpAddr, prefix: u8) -> bool {
    match (ip, base) {
        (IpAddr::V4(a), IpAddr::V4(b)) => {
            if prefix > 32 {
                return false;
            }
            let mask = if prefix == 0 {
                0u32
            } else {
                u32::MAX << (32 - prefix)
            };
            (u32::from(*a) & mask) == (u32::from(*b) & mask)
        }
        (IpAddr::V6(a), IpAddr::V6(b)) => {
            if prefix > 128 {
                return false;
            }
            let a = u128::from(*a);
            let b = u128::from(*b);
            let mask = if prefix == 0 {
                0u128
            } else {
                u128::MAX << (128 - prefix)
            };
            (a & mask) == (b & mask)
        }
        _ => false,
    }
}

fn genius_404() -> Response {
    crate::middleware::genius_404::genius_404_response()
}

pub async fn enforce_ip_allowlist(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let list = &state.config.ip_allowlist;
    // Empty list = open API (normal production for all players)
    if list.is_empty() {
        return next.run(req).await;
    }

    let path = req.uri().path();
    // Health always open so Render can probe
    if path == "/health" || path == "/health/store" {
        return next.run(req).await;
    }

    match client_ip(&req) {
        Some(ip) if ip_allowed(&ip, list) => next.run(req).await,
        Some(ip) => {
            tracing::warn!(target: "security", event = "ip_deny", %ip, path, "API blocked — IP not on allowlist");
            genius_404()
        }
        None => {
            tracing::warn!(target: "security", event = "ip_deny", path, "API blocked — could not determine client IP");
            genius_404()
        }
    }
}
