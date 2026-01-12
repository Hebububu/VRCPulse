# ==============================================================================
# VRCPulse - Production Dockerfile
# ==============================================================================

# Stage 1: Build
FROM rust:1.91-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libfontconfig1-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml migration/

# Create dummy files to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && mkdir -p migration/src && echo "" > migration/src/lib.rs

# Build dependencies only
RUN cargo build --release && rm -rf src migration/src

# Copy actual source code
COPY src ./src
COPY migration/src ./migration/src
COPY locales ./locales

# Touch main.rs to invalidate cache and rebuild
RUN touch src/main.rs

# Build release binary
RUN cargo build --release

# ==============================================================================
# Stage 2: Runtime
# ==============================================================================
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    fontconfig \
    fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/* \
    && fc-cache -fv

# Copy binary from builder
COPY --from=builder /app/target/release/vrc-pulse ./vrc-pulse

# Create data directory for SQLite volume
RUN mkdir -p /data

# Set default environment variables
ENV DATABASE_URL=sqlite:///data/vrcpulse.db?mode=rwc
ENV RUST_LOG=info,vrc_pulse=info

# Declare volume for persistent data
VOLUME ["/data"]

# Run the bot
CMD ["./vrc-pulse"]
