> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[DEVELOPMENT]` in audit logs.
>
> ### AI Assist Note
> 🛠 Tadpole OS: Developer Guide
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🛠 Tadpole OS: Developer Guide

Welcome to the Tadpole OS development ecosystem! This guide is designed to help you fork, modify, and contribute to the project, with a specific focus on **resource-constrained environments (8GB RAM / 30GB Disk)**.

---

## 🏗 System Architecture Overview

Tadpole OS is built using a modern 3-layer architecture:
1.  **Core Engine (`server-rs`)**: High-performance Rust backend using Axum and Tokyo.
2.  **Operations Dashboard (`src/`)**: React + Vite frontend with Zustand state management.
3.  **Deployment Swarm**: PowerShell/Bash scripts for multi-bunker orchestration.

---

## 🚀 Getting Started (Low-RAM Optimized)

If you are developing on a machine with **8GB RAM**, follow these steps to avoid system freezes during compilation.

### 1. Clone & Setup
```bash
git clone https://github.com/your-username/tadpole-os.git
cd tadpole-os
cp .env.example .env
```

### 2. Docker Development (Recommended)
Our `Dockerfile` is pre-configured with memory throttles. To start the environment:
```bash
docker compose up --build
```
> [!IMPORTANT]
> The first build compiles the entire Rust toolchain. On 8GB machines, this can take **15-20 minutes**. We have limited `CARGO_BUILD_JOBS=1` to prevent OOM (Out of Memory) errors.

### 3. Local "Split" Development (Fastest Iteration)
If you want to avoid Docker overhead during UI work:
- **Terminal 1 (Backend)**: `cd server-rs && cargo run`
- **Terminal 2 (Frontend)**: `npm run dev`

---

## 🐳 Docker Deployment & Multi-Arch

### Building for Apple Silicon & Linux
To build for multiple architectures (e.g., deploying from Intel to an M2 Mac or a Linux Bunker):
```bash
docker buildx build --platform linux/amd64,linux/arm64 -t tadpole-os:latest .
```

### Disk Usage (30GB Limit)
To keep your disk usage under the **30GB limit**:
- Run `docker system prune` regularly to remove old build layers.
- The `Dockerfile` uses a multi-stage approach; the final image is only **~150MB**, but the "Builder" layers can grow to 2GB+.

---

## 🌿 Contribution Workflow

1.  **Fork** the repository on GitHub.
2.  **Create a Feature Branch**: `git checkout -b feature/cool-new-ui`.
3.  **Commit with Purpose**: We follow [Conventional Commits](https://www.conventionalcommits.org/).
4.  **Verify**: Run `npm run lint` and `cargo test` before pushing.
5.  **Submit a PR**: Provide a clear description and screenshots of UI changes.

---

## 🧪 Testing

### Backend (Rust)
```bash
cd server-rs
cargo test
```

### Frontend (React/Playwright)
```bash
npm run test           # Unit tests
npx playwright test    # E2E tests
```

---

## 🎨 UI/UX Guidelines
- **Color Palette**: Use the curated HSL tokens in `tailwind.config.js`.
- **Animations**: Prefer CSS transitions or RAF-throttled animations for performance.
- **Responsiveness**: All cards must be draggable/resizable (see `LineageStream.tsx`).

---

## ❓ Need Help?
Check the `README.md` for project goals or open an Issue on GitHub for architectural clarification.

[//]: # (Metadata: [DEVELOPMENT])
