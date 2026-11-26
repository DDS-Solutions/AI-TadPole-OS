# --- Stage 1: Build the React Frontend ---
FROM node:24.15.0-slim AS frontend-builder

WORKDIR /usr/src/app

# Copy frontend dependency manifests
COPY package.json package-lock.json ./

# Install frontend dependencies (legacy-peer-deps to handle Vite 8 upgrade conflicts in plugins)
RUN npm ci --legacy-peer-deps

# Copy frontend source and configuration
COPY . .

# Build the frontend (produces /usr/src/app/dist)
RUN npm run build

# --- Stage 2: Build the Rust Backend ---
# Uses the official pinned Rust 1.95.0 toolchain image (eliminates curl|sh rustup download)
FROM rust:1.95.0-bookworm AS builder

# Prevent interactive prompts during apt-get
ENV DEBIAN_FRONTEND=noninteractive

# Build arguments for customization
ARG CARGO_BUILD_FLAGS=""
ARG EXTRA_RUSTFLAGS=""

# Install native build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    clang \
    lld \
    git \
    && rm -rf /var/lib/apt/lists/*

# Increase stack size and drastically limit jobs to save memory
ENV RUST_MIN_STACK=16777216
ENV CARGO_BUILD_JOBS=1
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV PROTOC=/usr/bin/protoc

WORKDIR /usr/src/app

# Copy agent configurations and embedded starter_kits fixtures for compile-time include_str! macros
COPY .agent/ ./.agent/
COPY starter_kits/ ./starter_kits/

# Copy the entire backend source
COPY server-rs/ ./server-rs/

# Build the real binary with strict memory limits
# We force codegen-units to 1 and disable entirely any Link-Time Optimization via rustflags
ENV CARGO_PROFILE_RELEASE_LTO=false
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
# ORT_STRATEGY=download is faster and compatible with Debian Bookworm
ENV ORT_STRATEGY=download
ENV RUSTFLAGS="-C lto=off -C opt-level=z -C debuginfo=0 -C link-arg=-fuse-ld=lld $EXTRA_RUSTFLAGS"

RUN cd server-rs && cargo build --target-dir /tmp/target --release $CARGO_BUILD_FLAGS

# --- Stage 3: Final Runtime Image (Ultra-lightweight, ABI-matched to Bookworm builder) ---
FROM debian:bookworm-slim

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /app

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    libgomp1 \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /tmp/target/release/server-rs /app/server-rs-bin
RUN chmod +x /app/server-rs-bin

# Copy the built React dashboard from the frontend-builder stage
COPY --from=frontend-builder /usr/src/app/dist /app/dist

# Copy data directory (skills, workflows, context, database)
COPY data /app/data

# Copy execution layer scripts
COPY execution /app/execution

# Copy agent configurations (mcp_config.json, skills, workflows)
COPY .agent /app/.agent

# Create writable runtime directories for agent sandboxes, state, and backups
RUN mkdir -p /app/workspaces /app/data/state /app/backups

# Create non-root user (UID 1001 to avoid conflict with default ubuntu user 1000)
# Keep binary, frontend dist, execution scripts, and .agent read-only (root:root)
RUN groupadd -g 1001 tadpole && useradd -r -u 1001 -g tadpole tadpole \
    && chown -R tadpole:tadpole /app/data /app/workspaces /app/backups

# Single port — Axum serves both API and frontend
EXPOSE 8000

USER tadpole

# Single process — no process manager needed
ENV STATIC_DIR=/app/dist

# Health check (TRAC-01 observability)
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/v1/engine/health || exit 1

CMD ["/app/server-rs-bin"]
