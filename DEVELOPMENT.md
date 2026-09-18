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

# Install NVIDIA SkillSpector locally for skill security audits
pip install skillspector
```

### 2. Docker Development (Recommended)
Our `Dockerfile` is pre-configured with memory throttles. To start the environment:
```bash
docker compose up --build
```
```

---

## 🎨 UI/UX Guidelines
- **Color Palette**: Use the curated HSL tokens in `tailwind.config.js`.
- **Animations**: Prefer CSS transitions or RAF-throttled animations for performance.
- **Responsiveness**: All cards must be draggable/resizable (see `LineageStream.tsx`).

---

## ❓ Need Help?
Check the `README.md` for project goals or open an Issue on GitHub for architectural clarification.