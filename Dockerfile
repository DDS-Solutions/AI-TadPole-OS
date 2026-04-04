# --- Stage 1: Build the React Frontend ---
FROM node:20-slim AS frontend-builder

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
FROM ubuntu:24.04 AS builder

# Prevent interactive prompts during apt-get
ENV DEBIAN_FRONTEND=noninteractive

# Build arguments for customization
ARG CARGO_BUILD_FLAGS=""
ARG EXTRA_RUSTFLAGS=""

# Install build dependencies and Rust
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    build-essential \
    protobuf-compiler \
    cmake \
    clang \
    lld \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Increase stack size and drastically limit jobs to save memory
ENV RUST_MIN_STACK=16777216
ENV CARGO_BUILD_JOBS=1
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV PROTOC=/usr/bin/protoc

WORKDIR /usr/src/app

# Copy the entire backend source
COPY server-rs/ ./server-rs/

# Build the real binary with strict memory limits
# We force codegen-units to 1 and disable entirely any Link-Time Optimization via rustflags
ENV CARGO_PROFILE_RELEASE_LTO=false
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
# ORT_STRATEGY=download is faster and usually works on Ubuntu 24.04
ENV ORT_STRATEGY=download
ENV RUSTFLAGS="-C lto=off -C opt-level=z -C debuginfo=0 -C link-arg=-fuse-ld=lld $EXTRA_RUSTFLAGS"

RUN cd server-rs && cargo build --target-dir /tmp/target --release $CARGO_BUILD_FLAGS

# --- Stage 3: Final Runtime Image (Ultra-lightweight) ---
FROM ubuntu:24.04

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /app

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /tmp/target/release/server-rs /app/server-rs-bin
RUN chmod +x /app/server-rs-bin

# Copy the built React dashboard from the frontend-builder stage
COPY --from=frontend-builder /usr/src/app/dist /app/dist

# Copy data directory (skills, workflows, context, database)
COPY data /app/data

# Create workspaces directory for agent sandboxes
RUN mkdir -p /app/workspaces

# Create non-root user (UID 1001 to avoid conflict with default ubuntu user 1000)
RUN groupadd -g 1001 tadpole && useradd -r -u 1001 -g tadpole tadpole
RUN chown -R tadpole:tadpole /app

# Single port — Axum serves both API and frontend
EXPOSE 8000

USER tadpole

# Single process — no process manager needed
ENV STATIC_DIR=/app/dist
CMD ["/app/server-rs-bin"]
