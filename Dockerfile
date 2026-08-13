# Genius Clan API — multi-stage build (Render free tier)
FROM rust:1.83-bookworm AS builder
WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release 2>/dev/null || cargo build --release \
    && rm -rf src

COPY backend/src ./src
RUN touch src/main.rs && cargo build --release

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
