# 📖 Tadpole OS Wiki Index
> **Catalog of the Sovereign Wiki (`AI-TadPole-OS`).**
Grouped by operational layer:

## 📋 Tier 1: Business Operations
- **[[Home]]** — Sovereign Wiki portal, Dual-Plane Sovereign Hierarchy (`IDENTITY.md v1.1.464`), and navigation map.
- **[[Getting-Started]]** — Prerequisites (Rust 1.91+ / 1.95.0, Node 22.22.2+/24.15.0+), native desktop installs, first launch, and dashboard connection.
- **[[Dashboard-Guide]]** — Multi-tab operations dashboard, detachable portals, keyboard shortcuts, and live telemetry HUD.
- **[[Chat-&-Voice]]** — Sovereign Chat, `/v1/engine/live-voice` (Privacy Shield gated), `/v1/engine/transcribe`, and `/v1/engine/speak`.
- **[[Projects-&-Missions]]** — Mission lifecycle, Conductor DAG planning, cluster workspaces, and Mission Flight Recorder dossiers.
- **[[AI-Tadpole-OS-Orchestration]]** — Topological task scheduling, sub-agent recruitment, and context sandboxing.
- **[[Daily-Workflows]]** — Synthetic SMB scenario walkthroughs (Plane 2 preview-only under FR-11).

## ⚙️ Tier 2: Systems Administration
- **[[Installation]]** — Source compilation, native installers, and Docker / Compose deployment prerequisites.
- **[[Configuration]]** — Authoritative `.env` reference including `NEURAL_TOKEN`, `NEURAL_ADMIN_TOKEN`, `OVERSIGHT_PUBLIC_KEY`, and backup/security flags.
- **[[LLM-Providers]]** — Local Ollama endpoints and cloud providers (OpenAI, Anthropic, Gemini, Groq, OpenRouter, DeepSeek).
- **[[Virtual-Employees]]** — Agent role profiles, model slot tuning, and Mythos reasoning parameters.
- **[[RAG-&-Memory]]** — LanceDB vector memory, Multi-Factor Scoring (MFS), deduplication, and SOP workflows.
- **[[A2A-Economics]]** — Two-Phase Commit (2PC) ledger, economic zones, x402 challenges, and outward A2A Gateway.
- **[[Updates-&-Backups]]** — Automated `TADPOLE_SNAP_V2` encrypted backup daemon (210k PBKDF2-HMAC-SHA512), 48h pruning, and restore procedures.

## 🔒 Tier 3: Security & Observability
- **[[Vault-&-Credentials]]** — Client-side PBKDF2 (600k) + AES-256-GCM Neural Vault, cross-tab lock sync, and in-memory step-up admin tokens.
- **[[Approvals-&-Quotas]]** — Admin-gated Oversight Queue, Ed25519 `ovs:v2` decision signing, Android hardware biometric `CryptoObject`, and budget caps.
- **[[Cost-Monitoring]]** — Real-time token accounting, latency benchmarks, and provider spend telemetry.
- **[[Kill-Switches]]** — Admin-gated `POST /v1/engine/kill`, `POST /v1/engine/resume`, `POST /v1/engine/shutdown`, and client Circuit Breakers.
- **[[Network-Boundaries]]** — CORS/Origin rules, WebSocket auth, Privacy Shield Live Voice gating, and egress map.
- **[[Parity-Guard]]** — 12-check automated drift detector (`execution/parity_guard.py`) and CI integration.

## 🛠️ General Documentation & Appendix
- **[[Developer-Appendix]]** — Core Rust/React architecture, dual-token RBAC, cross-platform deploy scripts, and Plane 1 lifecycle CLI.
- **[[Glossary]]** — Authoritative technical definitions and codebase implementation hooks.

---
*Last Updated: 2026-11-20 | [[Home]] | [Main Repo](https://github.com/DDS-Solutions/AI-TadPole-OS)*

[//]: # (Metadata: [wiki_index])
