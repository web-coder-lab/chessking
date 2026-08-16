# Genius Clan API — multi-stage (Render free tier)
FROM rust:1.97-bookworm AS builder
WORKDIR /app

# sqlx::migrate!("../database/migrations") → /database/migrations
COPY database /database
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/src ./src

# Release binary
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data

WORKDIR /app
COPY --from=builder /app/target/release/chess-king-backend /app/server

ENV PORT=8080
ENV DATABASE_URL=sqlite:///data/genius_clan.db
EXPOSE 8080
CMD ["/app/server"]
