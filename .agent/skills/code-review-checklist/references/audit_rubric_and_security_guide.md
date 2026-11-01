# Code Review Audit Rubric & Security Hardening Guide (L3 Reference)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review-checklist / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Inconsistent review standards, missed security vulnerabilities, ungrounded nitpicking.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document provides deep criteria for the 5 pillars of the Master Audit Rubric, including AI-native security, OWASP Agentic risks, and database optimization.

---

## 1. Deep Criteria by Rubric Pillar

### Pillar 1: Correctness & Logic
- **Boundary Conditions**: Are collections checked for emptiness before indexing? Are numbers checked for divide-by-zero or overflow?
- **Async Safety & Concurrency**: In Rust Tokio / Node.js, are shared states protected by appropriate mutexes/rwlocks? Are deadlocks possible from out-of-order lock acquisitions?
- **Cancellation & Cleanup**: When an async task or WebSocket connection aborts, are resources, file handles, and background listeners cleaned up?

### Pillar 2: Security & Sovereign Hardening
- **Input Validation**: Never trust external inputs (HTTP queries, IPC messages, websocket payloads). Validate against explicit JSON schemas or Rust structs.
- **SQL / Command Injection**: Use parameterized queries (`sqlx::query!`, prepared statements). Never concatenate raw strings into SQL or shell commands.
- **Secret Protection**: Verify zero hardcoded tokens, API keys, private keys, or passwords.
- **AI-Native & Agentic Security**:
  - **Prompt Injection Defense**: Treat all untrusted model inputs as potentially adversarial.
  - **Sink Sanitization**: Never pass model completions directly to system sinks (`eval`, shell execution, `innerHTML`, un-sandboxed filesystem writes).
  - **Capability Tokens**: Enforce Zero-Trust capability token verification on sensitive agent tool executions.

### Pillar 3: Performance & Scale
- **Algorithmic Complexity**: Eliminate $O(n^2)$ nested lookups across lists; convert to $O(1)$ HashSets or HashMaps.
- **Database & Query Efficiency**: Eliminate N+1 query patterns by using batch queries or SQL `JOIN`s.
- **Memory Footprint**: Avoid large in-memory buffers for file transfers; use streaming iterators or chunks.

### Pillar 4: Code Quality (`clean-code` Compliance)
- **Small Units**: Functions $\le 50$ lines; files $\le 500$ lines.
- **Guard Clauses**: Max 3 levels of indentation; early return on failure conditions.
- **Strict Typing**: Zero `any` in TypeScript; zero unhandled `.unwrap()` in production Rust.

### Pillar 5: Testing & Observability
- **Test Coverage**: Every new functional branch must have an accompanying unit or integration test.
- **Telemetry & Tracing**: Critical lifecycle events must emit structured telemetry tags (`[Subsystem]`, `[ERROR]`, `[OK]`).

---

## 2. Severity Classification & Triage Rules

| Severity | Definition | Action Required |
|---|---|---|
| 🔴 **BLOCKING** | Security vulnerability, data loss risk, compilation failure, unhandled crash, or severe logic break. | **Must be resolved before merge/ship.** |
| 🟡 **SUGGESTION** | Performance inefficiency, code smell, missing edge-case handling, or clean-code improvement. | **Strongly recommended; track if deferred.** |
| 🟢 **NIT** | Minor naming preference, comment clarification, or harmless aesthetic polish. | **Optional; author's discretion.** |
| ❓ **QUESTION** | Ambiguity in intent, missing context, or architectural trade-off needing discussion. | **Must be clarified.** |

---

## 3. Sample Concrete Review Audit Report

```markdown
### 🛡️ Code Review Audit Report

**Context**: Feature Update (Strict Rigor)  
**Target**: `src/routes/intelligence.rs`  
**Verdict**: 🔴 BLOCKING ISSUES FOUND

---

### Detailed Findings

1. 🔴 **BLOCKING** [Security]: Raw string concatenation in SQLite query at line 84.
   - *Risk*: SQL injection hazard via untrusted query parameters.
   - *Remedy*: Use `sqlx::query_as!(...)` with parameterized bind values.

2. 🟡 **SUGGESTION** [Performance]: Nested loop searching agent lists at line 120.
   - *Risk*: $O(n \times m)$ complexity degrades during large swarm missions.
   - *Remedy*: Pre-index agents into `HashMap<AgentId, Agent>` for $O(1)$ lookup.

3. 🟢 **NIT** [Clean Code]: Single-letter variable name `m` at line 142.
   - *Remedy*: Rename to `mission_summary`.

---

### Action Checklist
- [ ] [P0] Parameterize SQL query on line 84.
- [ ] [P1] Convert agent lookup to HashMap on line 120.
- [ ] [P2] Rename `m` to `mission_summary` on line 142.
```
