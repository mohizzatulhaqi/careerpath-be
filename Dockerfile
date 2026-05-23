# syntax=docker/dockerfile:1.7

# Stage 1: chef base — install cargo-chef once, reused by both stages
FROM rust:1.83-slim-bookworm AS chef
RUN cargo install cargo-chef --version 0.1.67 --locked
WORKDIR /app

# Stage 2: Planner — compute dependency recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder — cache deps, then build app
FROM chef AS builder

# Install build-time system deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cook dependencies — this layer is cached as long as recipe.json doesn't change
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy full source and build
COPY . .

# Use sqlx offline mode — requires .sqlx/ folder committed to repo
ENV SQLX_OFFLINE=true

RUN cargo build --release --bin career-path-be

# ============================================================
# Stage 4: Runtime — minimal image, binary only
# ============================================================
FROM debian:bookworm-slim AS runtime

# Install minimal runtime deps: TLS certs, libssl, curl (for healthcheck)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN useradd --create-home --shell /bin/bash --uid 1000 appuser

WORKDIR /app

# Copy compiled binary
COPY --from=builder /app/target/release/career-path-be /app/career-path-be

# Copy migrations (read at runtime by sqlx::migrate!)
COPY --from=builder /app/migrations /app/migrations

# Create storage directory with correct ownership
RUN mkdir -p /app/storage && chown -R appuser:appuser /app

USER appuser

HEALTHCHECK --interval=30s --timeout=10s --start-period=20s --retries=3 \
    CMD curl --fail --silent http://localhost:${SERVER_PORT:-3002}/health || exit 1

EXPOSE 3002

ENV RUST_LOG=info,career_path_be=info \
    SERVER_PORT=3002 \
    STORAGE_ROOT=/app/storage

CMD ["/app/career-path-be"]
