# Genius Clan API — free-tier friendly build
FROM rust:1.97-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY database /database
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src

# Cap parallelism + quieter lower memory pressure
ENV CARGO_BUILD_JOBS=1
ENV CARGO_INCREMENTAL=0
ENV CARGO_TERM_COLOR=never
RUN cargo build --release -j 1

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
