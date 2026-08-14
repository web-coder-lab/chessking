//! Branded Genius Clan 404 HTML — used for probes, IP deny, unknown API noise.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub const GENIUS_404_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>404 — Genius Clan</title>
<style>
  * { box-sizing: border-box; }
  body {
    margin: 0; min-height: 100vh; display: flex; align-items: center; justify-content: center;
    background: #0F1115; color: #F5F5F5;
    font-family: system-ui, -apple-system, Segoe UI, sans-serif;
    padding: 24px;
  }
  .card {
    max-width: 420px; width: 100%; text-align: center;
    background: #1A1D23; border-radius: 16px; border-top: 3px solid #D4AF37;
    padding: 40px 28px 32px;
  }
  .crown { font-size: 40px; line-height: 1; margin-bottom: 8px; }
  .brand {
    letter-spacing: 3px; text-transform: uppercase; font-size: 12px;
    color: #D4AF37; margin: 0 0 16px;
  }
  .code { font-size: 64px; margin: 0; font-weight: 700; color: #D4AF37; line-height: 1; }
  .title { font-size: 20px; margin: 12px 0 8px; }
  .body { color: #9CA3AF; font-size: 14px; line-height: 1.5; margin: 0 0 8px; }
</style>
</head>
<body>
  <div class="card">
    <div class="crown" aria-hidden="true">♚</div>
    <p class="brand">Genius Clan</p>
    <h1 class="code">404</h1>
    <p class="title">Page not found</p>
    <p class="body">This path doesn&apos;t exist on Genius Clan.</p>
  </div>
</body>
</html>"#;

pub fn genius_404_response() -> Response {
    (
        StatusCode::NOT_FOUND,
        [("content-type", "text/html; charset=utf-8")],
        GENIUS_404_HTML,
    )
        .into_response()
}
