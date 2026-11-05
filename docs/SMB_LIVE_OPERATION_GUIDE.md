> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: SMB Digital Twin / Live Operations & Production Readiness
> - **Architecture**: `@docs ARCHITECTURE:Core`
> - **Failure Path**: Premature live data processing, cross-tenant data leakage, unverified external side effects, or shell privilege escalation.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SMB_LIVE_OPERATIONS]`)

# 🏢 SMB Digital Twin: Live Operations & Production Readiness Guide

> **Target Audience**: Small & Medium Business (SMB) Founders, System Operators, Compliance Leads, and Technical Architects.  
> **Status**: Governance Directive & Operational Roadmap  
> **Sovereign Authority**: [`directives/IDENTITY.md`](../directives/IDENTITY.md) (`Plane 2: Company-Agnostic SMB AI Digital Twin`, `FR-11` Permission Firewall)  
> **Technical Specification**: [`docs/specs/mock-smb-digital-twin.md`](./specs/mock-smb-digital-twin.md)  
> **Compliance & Invariant Standard**: ADG v2, GxP, ISO 42001, NIST AI-RMF  

---

## 🛑 The Core Boundary: Preview vs. Live Operations

> [!WARNING]
> **Plane 2 Is Preview-Only in the Generic Runner**:  
> In the standard `AI-Tadpole-OS` distribution, **Plane 2 (SMB Digital Twin) is strictly restricted to synthetic previews**.  
> The generic Rust runner enforces a fail-closed gate: identities whose stored department matches `Plane 2: SMB-Digital-Twin` are blocked from task dispatch.  
> **Never import, persist, publish, or execute real customer communications, financial transactions, or PII through the generic runner.**

To take an SMB from synthetic experimentation to real-world, autonomous business operations, the system must undergo a structured transition across **6 Technical Shifts**. This guide documents what must change, why it must change, and how operators can verify readiness.

---

## 🗺️ Dual-Plane Architectural Boundary

```mermaid
graph TD
    User([SMB Operator / Entity 0]) --> Gateway[Sovereign Intent Gateway]
    
    Gateway -->|Plane 1: Codebase Maintenance| P1Runner[Generic Rust Runner]
    Gateway -->|Plane 2: SMB Business Twin| Guard{{"FR-11 Security Firewall<br/>(Fail-Closed Admission Gate)"}}
    
    subgraph "Plane 1: Sovereign Codebase Engine (Active Today)"
        P1Runner --> CodebaseTools[AST Graph / File IO / Shell / Git / Compiler]
        CodebaseTools --> EngineState[(Engine SQLite / Workspaces)]
    end
    
    subgraph "Plane 2: SMB AI Digital Twin (Planned Live Runtime)"
        Guard -->|Preview / Synthetic| MockEnv[Practice Companies: Mock-1 / Mock-2]
        MockEnv --> MockConnectors[Synthetic Mocks & Scenarios]
        
        Guard -.->|Live Switch (Requires 6 Shifts)| LiveRuntime[Company-Scoped Live Runtime]
        LiveRuntime --> LiveConnectors[Certified Connectors: ERP / CRM / Banking]
        LiveRuntime --> CompanyVault[(Isolated Company Storage & Vault)]
    end
```

---

## ⚙️ The 6 Technical Shifts Required for Live Operations

When transitioning from the preview sandbox to a live business node, the following six architectural pillars must be implemented:

### 1. Company-Scoped Storage & Execution Isolation (`FR-5`, `FR-6`, `NFR-1`)
* **Current State (Preview)**: Agents operate in a shared local SQLite database and write to shared workspace directories under `data/workspaces/`.
* **Live Requirement**:
  - Every live business run must be cryptographically and transactionally bound to a 4-tuple context:
    $$\text{Context} = (\text{company\_key}, \text{workspace\_id}, \text{environment: 'live'}, \text{run\_id})$$
  - Tenant databases must be completely segregated. A company key prefix alone is **never** an isolation boundary.
  - Cross-tenant reads, writes, memory queries, and approval reuse must be blocked at the database engine level.

### 2. Connector Promotion: Synthetic Mocks to Certified Live APIs (`FR-8`, `FR-10`)
* **Current State (Preview)**: Skills use synthetic data fixtures and mock business connectors (`Mock-1-Product`, `Mock-2-Service`).
* **Live Requirement**:
  - Connectors to real business systems (QuickBooks, Stripe, HubSpot, Salesforce, Gmail, Shopify) must be certified.
  - Every connector must implement:
    1. **Strict Idempotency**: Safe retry semantics preventing duplicate invoice generation or multiple email dispatches.
    2. **Deterministic Receipts**: Cryptographic receipts verifying the external record ID and response timestamp.
    3. **3-Phase Promotion Lifecycle**:
       $$\text{Phase 1: Synthetic Mock} \longrightarrow \text{Phase 2: Read-Only Shadow} \longrightarrow \text{Phase 3: Approved Live Action}$$

### 3. Hardened Dual-Plane Permission Firewall (`FR-11`)
* **Current State (Preview)**: The generic runner checks `agent.department.eq_ignore_ascii_case("Plane 2: SMB-Digital-Twin")`.
* **Live Requirement**:
  - Business agents representing virtual employees (Support Analyst, Bookkeeper, Sales Lead) must **never** inherit system engineering privileges.
  - Under no circumstances may a Plane 2 agent access:
    - Code execution or terminal shells (`execute_shell_command`).
    - Repository modifications (`write_file`, `git commit`).
    - Internal engine databases or raw secrets.
  - Capabilities must be constrained strictly to authorized MCP business tools and external connector endpoints.

### 4. Enterprise Key Management & Secret Isolation
* **Current State (Preview)**: API keys and credentials reside in the browser-local Neural Vault (PBKDF2/AES-GCM) or local `.env` files.
* **Live Requirement**:
  - Live production requires hardware-backed KMS (e.g., AWS KMS, HashiCorp Vault, or TPM-backed secure enclaves).
  - Credentials (OAuth tokens for accounting, banking, email) must be dynamically fetched per company run and automatically redacted from application traces and logs.

### 5. Verifiable Audit Evidence & Human-in-the-Loop Oversight (`FR-9`, `NFR-3`)
* **Current State (Preview)**: Missions complete based on the agent's synthesized text response.
* **Live Requirement**:
  - An agent's narrative claim ("I sent the quote to the customer") is **unverified** without third-party proof.
  - High-risk operations (wire transfers, contract dispatch, payroll calculations) must enforce:
    - **Dual-Party Human Approval**: Mandatory prompt in the Oversight Dashboard requiring manual operator sign-off.
    - **Immutable Merkle Audit Log**: Every action linked cryptographically to the mission flight recorder.

### 6. Streaming PII Scrubbing & Data Governance
* **Current State (Preview)**: PII skills advise against pasting real customer data.
* **Live Requirement**:
  - An automated client-side streaming PII redaction layer must intercept all ingested customer emails, tickets, and documents before they are dispatched to LLM inference providers.
  - Zero Data Retention (ZDR) agreements must be active for cloud inference endpoints, or inference must be strictly routed to local self-hosted nodes (e.g., Ollama running Gemma 4 / Qwen).

---

## 📋 The Operator's Migration Checklist (The "Live Switch")

Before flipping any company configuration from `mock` to `live`, the SMB operator must complete this formal readiness checklist:

| Phase | Milestone | Verification Command / Metric | Status |
| :--- | :--- | :--- | :---: |
| **1. Identity** | Assign registered `company_key` and unique tenant workspace | Validated via `GET /v1/workspaces/{id}` | 🔲 |
| **2. Calibration** | Calibrate Digital Twin tone using synthetic samples | Run `.agent/skills/digital-twin-voice/scripts/voice_calibrator.py` | 🔲 |
| **3. Acceptance** | 100% pass on 8 mock scenarios (enquiry, stale facts, duplicates) | Execute `docs/specs/mock-smb-digital-twin.md` Acceptance Suite | 🔲 |
| **4. Connectors** | Run connectors in **Read-Only Shadow Mode** for $\ge 7$ business days | Confirm 0 unhandled connector timeouts or schema drifts | 🔲 |
| **5. Oversight** | Configure approval thresholds in `Oversight Dashboard` | Financial limits set ($> \$100$ requires Entity 0 confirmation) | 🔲 |
| **6. PII Shield** | Verify automated redaction on synthetic customer sample runs | Invariant `tests/invariants/inv_003_telemetry_scrub.test.ts` passing | 🔲 |
| **7. Production** | Promote environment toggle: `environment: "live"` | Server authorizes company key and mounts isolated live DB | 🔲 |

---

## 📍 Where the Boundary is Referenced in the Codebase

For auditing, compliance, and developer verification, the Plane 2 preview boundary is maintained across these key anchors:

1. **Governance Authority**: [`directives/IDENTITY.md`](../directives/IDENTITY.md) — *Sections: System Identity, Dual-Plane Topology & Directive #1*.
2. **Technical Specification**: [`docs/specs/mock-smb-digital-twin.md`](./specs/mock-smb-digital-twin.md) — *Full architectural blueprint and ubiquitous domain terminology*.
3. **Fail-Closed Runner Gate**: [`server-rs/src/routes/agent/missions.rs`](../server-rs/src/routes/agent/missions.rs) — *FR-11 admission filter rejecting Plane 2 mission dispatches*.
4. **Operations Manual**: [`docs/OPERATIONS_MANUAL.md`](./OPERATIONS_MANUAL.md) — *Business Operations Tier 1-3 terminology and preview notices*.
5. **Voice Calibration Skill**: [`.agent/skills/digital-twin-voice/SKILL.md`](../.agent/skills/digital-twin-voice/SKILL.md) — *Preview-only tone guidance protocol*.
6. **Data Ingestion Skill**: [`.agent/skills/smb-data-ingestion/SKILL.md`](../.agent/skills/smb-data-ingestion/SKILL.md) — *Synthetic data verification guidelines*.
7. **PII Redaction Skill**: [`.agent/skills/pii-redaction/SKILL.md`](../.agent/skills/pii-redaction/SKILL.md) — *Zero-trust redaction instructions*.

---

## 📞 Support & Enterprise Hardening

For SMBs seeking enterprise multi-tenant licensing, hardware-enclave installation, or custom ERP connector certification, refer to:
- **Repository Issues**: [GitHub Issues](https://github.com/DDS-Solutions/AI-TadPole-OS/issues)
- **Security Disclosures**: [`docs/SECURITY.md`](./SECURITY.md)

