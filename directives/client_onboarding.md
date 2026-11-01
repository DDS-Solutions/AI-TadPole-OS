> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / client_onboarding
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[client_onboarding]`)

# 🤝 Directive: Client Onboarding (SOP-HR-03)

## 🎯 Primary Objective
Provide a seamless, "White-Glove" entry experience for organizations evaluating AI-Tadpole-OS. Plane 2 onboarding is currently a planning and preview flow; company-scoped runtime implementation is pending.

> **Plane 2 stop condition**: The generic Rust runner rejects only identities whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not trigger the guard. Until company-scoped authorization and storage exist, use synthetic data and stop at preview. Do not import, persist, publish, or execute live customer workflows through the generic runner.

> 💡 **Bound Skills (Plane 2 — `SMB-Digital-Twin`)**:
> - Use `@[skills/digital-twin-voice]` to draft the founder's brand tone and communication persona for preview; do not persist it in a shared workspace.
> - Use `@[skills/smb-data-ingestion]` to validate messy SMB exports and produce a preview/quarantine report; do not import records into storage yet.
> - Use `@[skills/pii-redaction]` as redaction guidance; the skill alone does not guarantee all pipeline inputs or public catalog data are scrubbed.
> - Use `@[skills/wayfinder]` to map multi-session transformation epics (`Destination`, `Decisions so far`, `Not yet specified`, `Out of scope`).
> - Use `@[skills/to-spec]` to draft non-sensitive, company-agnostic requirements for operator review; do not place client records or identifying details in repository specs.

> [!IMPORTANT]
> **🛡️ `FR-11` requirement, not completed onboarding** *(IDENTITY.md Directive #1)*: Do not provision or run business agents through the generic runner. Its fail-closed check recognizes only the exact stored department `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not activate it. Practice workspaces and promotion to live company data remain requirements in [`docs/specs/mock-smb-digital-twin.md`](../docs/specs/mock-smb-digital-twin.md), not current capabilities.

---

## 🏗️ Onboarding Roadmap

### 1. Workspace Readiness (Preview Only)
- **Action**: Record the proposed company key and workspace requirements; do not treat a generic `cluster_id` as a company boundary or create a live-data workspace.
- **Check**: Confirm the company-scoped runtime is still pending; use synthetic inputs and stop at preview.

### 2. Identity Forge
- **Role Assignment**: Draft proposed roles for review. Do not provision company-facing agents in the generic runner while Plane 2 is pending.
- **Authentication**: Walk the user through the `Neural Vault` setup and key injection.

### 3. Governance Tuning
- **Budget**: Initialize the `DEFAULT_AGENT_BUDGET_USD` and set up the `Budget Guard` thresholds.
- **Privacy**: Set `PRIVACY_MODE` only for runner model-routing policy. It does not create an air gap and the global monitor sends periodic reachability probes; enforce client egress requirements with host/network controls.

---

## 🛠️ Initial Mission SOP

### 1. The "First Light" Task
- **Action**: Assign a simple research task (e.g., "Audit local documentation") to the new cluster.
- **Goal**: Verify end-to-end communication from the UI to the Engine and back.

### 2. Walkthrough Generation
- **Deliverable**: `reports/ONBOARDING_WALKTHROUGH.md`. Include specific instructions for using the `Sovereign Dashboard`.

---

## 🚦 Success Criteria
Plane 1 engine onboarding may be verified separately. Plane 2 business onboarding is not complete until company-scoped storage, authorization, and runtime controls are implemented and verified; a preview does not count as a completed business run.
