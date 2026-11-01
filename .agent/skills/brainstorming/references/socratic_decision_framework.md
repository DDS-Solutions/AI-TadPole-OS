# Socratic Decision Framework & Questioning Architecture (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / brainstorming / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Premature implementation, unverified assumptions, conversational drift.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This reference document defines the formal decision framework, ambiguity scoring matrix, and domain-specific decision banks for the Sovereign Socratic Gate.

---

## 1. Ambiguity Scoring & Socratic Gate Activation

Before any code modification or file generation, the agent must evaluate the request against the **Ambiguity Scoring Matrix**:

| Factor | Trigger Condition | Points |
|---|---|---|
| **Vague Verbs** | "Add", "Build", "Update", "Optimize", "Integrate" without target file or API contract | +2 |
| **Architectural Fork** | Multiple viable design approaches exist with divergent cost/complexity | +2 |
| **Missing Constraints** | No declared latency, scale, security, or error tolerance criteria | +1 |
| **Multi-File Impact** | Changes span 3 or more modules or cross the frontend/backend boundary | +1 |
| **Irreversible State** | Involves schema migrations, database deletion, or external credentials | +2 |

### Evaluation Threshold:
- **Total Points $\ge 3$**: **Socratic Gate Triggered (MANDATORY HALT).** The agent must stop, perform memory check, and pose structured P0/P1 questions. No implementation code may be generated.
- **Total Points $< 3$**: Gate is bypassed. The agent proceeds to `ORCHESTRATE` or `IMPLEMENT`.

---

## 2. Decision Hierarchy & Priority Classification

Every question must be classified by priority to prevent overwhelming the user:

```
                      ┌─────────────────────────┐
                      │    P0: BLOCKING GATES   │
                      │  (Cannot code without)  │
                      └────────────┬────────────┘
                                   │
                      ┌────────────▼────────────┐
                      │ P1: ARCHITECTURAL FORKS │
                      │  (Affects >30% of code) │
                      └────────────┬────────────┘
                                   │
                      ┌────────────▼────────────┐
                      │   P2: DEFERRABLE OPTS   │
                      │ (Can adopt defaults now)│
                      └─────────────────────────┘
```

1. **P0 (Blocking Gates)**:
   - If unanswered, implementation is physically blocked or completely arbitrary.
   - Maximum allowed in a single turn: **2 questions**.
2. **P1 (Architectural Forks)**:
   - High-leverage structural choices (e.g., synchronous vs asynchronous, push vs pull).
   - Maximum allowed: **2 questions**.
3. **P2 (Deferrable Options)**:
   - Edge cases, polish, styling variants. Always provide a sensible default recommendation.

---

## 3. Sovereign Domain Decision Banks

### A. IPC & System Transport
| Decision Fork | Option A | Option B | Default Rationale |
|---|---|---|---|
| **Frontend-to-Core Bridge** | Tauri IPC Command (`tauri::command`) | Axum Local HTTP / WebSocket | Tauri IPC for low-latency native calls; Axum when streaming large payloads or multi-client |
| **Async Communication** | Tokio Broadcast Channel | Crossbeam / MPSC FIFO | Tokio Broadcast when multiple workers subscribe; MPSC for dedicated single-consumer queue |

### B. Persistence & State Storage
| Decision Fork | Option A | Option B | Default Rationale |
|---|---|---|---|
| **Structured Metadata** | SQLite via SQLx / rusqlite | In-Memory Zustand Store | SQLite for persistent OS configurations and credentials; Zustand for reactive UI session state |
| **Vector Embeddings** | Embedded LanceDB | Remote Vector Service (Pinecone/Qdrant) | Embedded LanceDB for Sovereign local-first privacy; Remote only if explicitly mandated by user |

### C. Security & Privilege Boundaries
| Decision Fork | Option A | Option B | Default Rationale |
|---|---|---|---|
| **Execution Sandboxing** | Zero-Trust capability token required | Host OS direct execution | Zero-Trust capability token for third-party scripts; Direct execution only for verified internal tools |
| **Secret Management** | PBKDF2/AES-GCM encrypted vault | Local environment variable (.env) | Vault storage for user credentials and API keys; .env for static runtime port configurations |

---

## 4. Compliant Socratic Proposal Schema

When halting at the Socratic Gate, format the proposal using this schema:

```markdown
### 🛑 Socratic Gate: Alignment Required

I have evaluated your request to **[Goal]**. Before proceeding to implementation, we must resolve **[N]** key architectural decision points:

---

### [PRIORITY: P0] **[Decision Name]**

**Question**: [Direct, unambiguous question]

**Architectural Impact**:
- [Consequence on performance, complexity, or security]
- [Affects: Cost / Maintenance / Scalability]

**Trade-Off Matrix**:
| Option | Pros | Cons | Recommended For |
|---|---|---|---|
| **A: [Name]** | [Advantage] | [Drawback] | [Target scenario] |
| **B: [Name]** | [Advantage] | [Drawback] | [Target scenario] |

**Default Recommendation**: Option [A/B] because [Concrete architectural rationale].

---

### Suggested Execution Path:
If you agree with the default recommendations, reply with **"Proceed"** or indicate your preferred options to begin orchestration.
```

---

## 5. Transition to Next Lifecycle Stages

```
[BRAINSTORMING: Socratic Gate]
             │
             ▼ (User confirms options / "Proceed")
[ORCHESTRATE: Decomposition into task.md]
             │
             ▼ (Steps planned)
[ALETHEIA: Invariant verification]
             │
             ▼ (Logic verified)
[IMPLEMENT: Clean-code execution]
```

At no point may the agent bridge directly from `BRAINSTORMING` to `IMPLEMENT` without explicit user confirmation of the architectural path.
