> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[ROADMAP]` in audit logs.
>
> ### AI Assist Note
> 🗺️ Tadpole OS Roadmap
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🗺️ Tadpole OS Roadmap

Tadpole OS is evolving from a high-performance local engine into a fully autonomous, sovereign multi-agent swarm platform. This roadmap outlines our journey through strategic hardening and advanced capability expansion.

## 🟢 Phase 1: Core Engine & Sovereign Intelligence (Current)
- [x] **High-Performance Rust Backend**: Axum-based engine with lock-step phase transitions.
- [x] **Local-First Memory**: LanceDB vector store with Gemini `text-embedding-004`.
- [x] **Reactive Dashboard**: Real-time telemetry and Agent Management UI.
- [x] **Modular Skills (MCP)**: Native Support for Model Context Protocol.
- [x] **Voice Synthesis/Recognition**: Piper STT/TTS with Bunker caching.

## 🟡 Phase 2: Autonomous Continuity & Swarm Governance
- [x] **Intelligent Model Registry & Capability Sync** `IMR-01`: Dynamic provider model discovery (Ollama, OpenAI, Groq, Google, etc.) with automatic capability enrichment (tool support, vision, context window, embedding detection). Enables UI-level enforcement and intelligent swarm routing. Full local/self-hosted model support. → [Spec](directives/features/intelligent-model-registry.md)
- [ ] **Continuity Scheduler**: Background swarm execution with budget-capped cron jobs.
- [ ] **Unified Oversight Gate**: Strategic approval layer for sensitive tool calls (Shell, Budget).
- [ ] **GraphRAG Integration**: Precise entity-relation retrieval via context graphs (TrustGraph-inspired).
- [ ] **Flow-based Ingestion**: Asynchronous data processing pipelines for high-volume context.
- [ ] **Industry Template Ecosystem**: "One-Click" deployment for specialized agent clusters.
- [ ] **Sovereign STT Improvements**: Higher precision Whisper integration.
- [ ] **Unified Communication Bridge**: Real-time notifications and HITL via Slack/Discord MCP.
- [ ] **Swarm Memory Deduplication**: Cross-mission pattern recognition to reduce token cost.

## 🟡 Phase 3: High-Fidelity Observability & Distributed Swarms
- [x] **Advanced Swarm Observability**: Industry-standard telemetry (Observe-Call-Audit) for mission auditing.
- [x] **Swarm Collaboration Infrastructure**: Persistent directives, hierarchical delegation, and peer-led audit loops.
- [x] **Global Swarm Memory (Vault)**: Cross-mission intelligence sharing via persistent vector store.
- [x] **Autonomous Skill Synthesis**: Agent-driven tool creation and dynamic registry enrollment.
- [ ] **P2P Swarm Network**: Multi-node coordination across local networks (Bunker Mesh).
- [ ] **Context Cores**: Portable, versioned knowledge artifacts for cross-mission portability.
- [ ] **Graceful Degradation Protocols**: Amber/Red badging for provider failover.
- [ ] **Self-Annealing Operations**: Autonomous recovery from failed tool calls.
## 🟢 Phase 4: Autonomous Evolution & Swarm Mesh (Complete)
- [x] **Global Intelligence Loop (RAG)**: Automated context retrieval from the Global Swarm Vault.
- [x] **Autonomous Tool Refinement**: `refactor_synthesized_skill` for self-patching capabilities.
- [x] **Dynamic Registry Refresh**: Hot-reloading of agent-generated skills without restart.
- [x] **Evolution Telemetry**: Real-time tracking of skill synthesis and refinement history.

## 🔴 Phase 5: Resilient Infrastructure & Security Hardening
- [x] **Secret-Aware Telemetry**: Automated redaction of API keys and bearer tokens in logs.
- [ ] **Granular Backend Errors**: RFC 9457 compliance with machine-readable `ErrorCode` for UI remediation.
- [ ] **Automated Trace Correlation**: Zero-config `X-Request-Id` injection for 100% request-lifespan tracking.
- [ ] **Institutional Knowledge Store**: Cross-cluster semantic persistent memory.

## 🚀 Future Vision
- **Hardware-Agnostic Acceleration**: Optimized for NVIDIA, ARM (Mac), and RISC-V.
- **Tadpole Hub**: Decentralized registry for public agent templates.
- **Deep Web Integration**: Native SERP and browsing clusters for truly autonomous research.
- **Universal Sovereign Ingestion**: Native multi-modal document ingestion (PDFs, Office files, Images) utilizing MarkItDown to convert unstructured corporate assets into agent-native Markdown for enhanced RAG and Swarm context.

---

## 📋 Feature Specs
Detailed feature specifications live in [`directives/features/`](directives/features/). Each spec is linked from its roadmap item and contains full architecture, implementation plan, acceptance criteria, and files affected.

| Feature ID | Name | Phase | Status |
|---|---|---|---|
| `IMR-01` | [Intelligent Model Registry & Capability Sync](directives/features/intelligent-model-registry.md) | Phase 2 | ✅ Completed |
| `IMR-02` | Dynamic Capability Overrides | Phase 2 | Planned |

---
*Last Updated: May 1, 2026*

[//]: # (Metadata: [ROADMAP])
