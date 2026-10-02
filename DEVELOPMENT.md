> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / DEVELOPMENT
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[DEVELOPMENT]`)

# 🛠 Tadpole OS: Developer Guide

Welcome to the Tadpole OS development ecosystem. This guide helps you fork, modify, and contribute — including on **resource-constrained** machines (≈8GB RAM / 30GB disk).

---

## 🏗 System Architecture Overview

1. **Core Engine (`server-rs`)** — High-performance Rust backend using Axum and **Tokio**.
2. **Operations Dashboard (`src/`)** — React + Vite frontend with Zustand state management.
3. **Deployment helpers** — PowerShell/Bash scripts for packaging and Linux host install (see `docs/DEPLOYMENT_GUIDE.md`).

### Toolchain pins (tip)

| Layer | Pin |
| :--- | :--- |
| Rust | `rust-toolchain.toml` → **`stable`** (CI uses stable; do not assume a frozen `1.85` MSRV badge) |
| Node | `package.json` engines → **^22.22.2+** (also 24.x / 26+) |
| Frontend | React **^19.3**, Vite **^8.3**, Tailwind **^4.3** |

---

## 🚀 Getting Started (Low-RAM Optimized)

### 1. Clone & Setup

```bash
git clone https://github.com/DDS-Solutions/AI-TadPole-OS.git
cd AI-TadPole-OS
cp .env.example .env
# Set NEURAL_TOKEN at minimum. See docs/GETTING_STARTED.md for ADMIN/CAPABILITY/PRIVACY.
npm install
```

Optional: `pip install skillspector` for NVIDIA SkillSpector audits.

### 2. Local dual-process (recommended for active development)

```bash
# Terminal A — Rust engine (:8000)
npm run engine

# Terminal B — Vite dashboard (:5173)
npm run dev
```

### 3. Docker / GHCR (runtime node, not a full rebuild)

`docker-compose.yml` uses the **prebuilt** image `ghcr.io/dds-solutions/ai-tadpole-os:latest` — there is **no** `build:` section. Prefer:

```bash
docker compose up -d
```

Use `docker compose up --build` only if you intentionally add a local build service; stock compose will not rebuild the engine from this Dockerfile.

For a from-source container image, see the root `Dockerfile` and CI publish workflows.

---

## 🎨 UI/UX Guidelines

- **Color palette**: curated HSL tokens / Tailwind theme variables.
- **Animations**: CSS transitions or RAF-throttled animations.
- **Responsiveness**: cards that are draggable/resizable (see `LineageStream.tsx`).

---

## ❓ Need Help?

- Operator onboarding: `docs/GETTING_STARTED.md`
- Deploy: `docs/DEPLOYMENT_GUIDE.md`
- Project goals: `README.md` — or open a GitHub Issue for architectural clarification.
