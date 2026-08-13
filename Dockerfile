# Genius Clan API — multi-stage build (Render free tier)
FROM rust:latest AS builder
WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src \
    && find target/release -name 'chess-king*' -delete 2>/dev/null || true

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
