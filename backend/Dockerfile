# Genius Clan — API (Rust / Axum) — Render free-tier friendly multi-stage build
FROM rust:1.83-bookworm AS builder
WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
COPY ../database /database
# Re-touch so cargo rebuilds with real sources
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/chess-king-backend /app/server
# SQLite file lives on a Render disk (Phase 3) or ephemeral for Phase 1
ENV PORT=8080
ENV DATABASE_URL=sqlite:///data/genius_clan.db
EXPOSE 8080

# Create data dir for SQLite
RUN mkdir -p /data

CMD ["/app/server"]
