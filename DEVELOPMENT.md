> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / DEVELOPMENT
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[DEVELOPMENT]`)

# 🛠 Tadpole OS: Developer Guide

Welcome to the Tadpole OS development ecosystem! This guide is designed to help you fork, modify, and contribute to the project, with a specific focus on **resource-constrained environments (8GB RAM / 30GB Disk)**.

---

## 🏗 System Architecture Overview

Governed by the **Dual-Plane Sovereign Hierarchy** ([`directives/IDENTITY.md`](directives/IDENTITY.md)), Tadpole OS supports **Plane 1: Sovereign Codebase Lifecycle Engine** and documents a planned **Plane 2: Company-Agnostic SMB AI Digital Twin**. Plane 2 workflows, company-scoped authorization, and isolated storage are not implemented; use synthetic inputs and stop at preview. The current exact department-marker rejection is not full **`FR-11`** enforcement.

Tadpole OS is built using a modern 3-layer architecture:
1.  **Core Engine (`server-rs`)**: High-performance Rust backend using Axum and Tokio.
2.  **Operations Dashboard (`src/`)**: React + Vite frontend with Zustand state management.
3.  **Deployment Swarm (`directives/` & `execution/`)**: Deterministic SOPs, Python execution scripts, and multi-bunker orchestration.

---

## 🚀 Getting Started (Low-RAM Optimized)

If you are developing on a machine with **8GB RAM**, follow these steps to avoid system freezes during compilation.

### 1. Clone & Setup
```bash
git clone https://github.com/DDS-Solutions/AI-TadPole-OS.git
cd AI-TadPole-OS
cp .env.example .env

# Install NVIDIA SkillSpector locally for skill security audits
pip install skillspector
```

### 2. Docker Development (Recommended)
Our `Dockerfile` is pre-configured with memory throttles (`CARGO_PROFILE_RELEASE_LTO=false`, `CODEGEN_UNITS=1`). To build locally or run the pinned Compose stack (ensure `.env` defines distinct `NEURAL_TOKEN`, `NEURAL_ADMIN_TOKEN`, `OVERSIGHT_PUBLIC_KEY`, `GF_ADMIN_PASSWORD`, and `.secrets/prometheus-token` exists):
```bash
# Build local image from Dockerfile
docker build -t ghcr.io/dds-solutions/ai-tadpole-os:local .

# Or start the pinned Compose observability + engine stack
docker compose up -d
```

---

## 🎨 UI/UX Guidelines
- **Color Palette**: Use the curated Tailwind v4 `@theme` design tokens in `src/index.css` (no `tailwind.config.js`).
- **Animations**: Prefer CSS transitions or RAF-throttled animations for performance.
- **Responsiveness**: All cards must be draggable/resizable (see `src/components/Lineage_Stream.tsx`).

---

## ❓ Need Help?
Check the `README.md` for project goals or open an Issue on GitHub for architectural clarification.

## Sidecar and protoc binaries

Do not commit `src-tauri/bin/server-rs-*.exe` or vendored `protoc.exe` copies. Build the engine with `cargo build --release --manifest-path server-rs/Cargo.toml` and copy the product into `src-tauri/bin/` using the Tauri sidecar name before `tauri build`. Install `protoc` from the official Protocol Buffers release and set `PROTOC` if a build script requires it.

