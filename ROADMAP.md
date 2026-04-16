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
- [ ] **P2P Swarm Network**: Multi-node coordination across local networks (Bunker Mesh).
- [ ] **Context Cores**: Portable, versioned knowledge artifacts for cross-mission portability.
- [ ] **Graceful Degradation Protocols**: Amber/Red badging for provider failover.
- [ ] **Self-Annealing Operations**: Autonomous recovery from failed tool calls.
## 🔴 Phase 4: Resilient Infrastructure & Security Hardening
- [ ] **Secret-Aware Telemetry**: Automated redaction of API keys and bearer tokens in logs.
- [ ] **Granular Backend Errors**: RFC 9457 compliance with machine-readable `ErrorCode` for UI remediation.
- [ ] **Automated Trace Correlation**: Zero-config `X-Request-Id` injection for 100% request-lifespan tracking.
- [ ] **Institutional Knowledge Store**: Cross-cluster semantic persistent memory.

## 🚀 Future Vision
- **Hardware-Agnostic Acceleration**: Optimized for NVIDIA, ARM (Mac), and RISC-V.
- **Tadpole Hub**: Decentralized registry for public agent templates.
- **Deep Web Integration**: Native SERP and browsing clusters for truly autonomous research.

---

## 📋 Feature Specs
Detailed feature specifications live in [`directives/features/`](directives/features/). Each spec is linked from its roadmap item and contains full architecture, implementation plan, acceptance criteria, and files affected.

| Feature ID | Name | Phase | Status |
|---|---|---|---|
| `IMR-01` | [Intelligent Model Registry & Capability Sync](directives/features/intelligent-model-registry.md) | Phase 2 | ✅ Completed |
| `IMR-02` | Dynamic Capability Overrides | Phase 2 | Planned |

---
*Last Updated: April 2026*
