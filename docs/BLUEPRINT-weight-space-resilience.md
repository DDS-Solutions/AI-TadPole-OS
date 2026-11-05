//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Architecture / Blueprints
//! - **Feature**: Weight-Space Resilience & Attestation Suite
//! - **Status**: APPROVED_BLUEPRINT
//! - **Reference**: `https://rikiyat.github.io/blog/weight-space-programming/`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Zero-DB-Drift]` In-memory runner state for Phase 1; existing `capability_proposals` schema for Phase 3 (no new migrations).
//! - `[Zero-Token-Bloat]` Do not solve loops with prompt inflation; enforce action-space masking and monologue pruning.
//! - `[Zero-Trust-Preservation]` Tool masking is strictly subtractive; `weights:edit` requires mandatory HITL oversight.
//! - `[Dual-Plane-Isolation]` Weight inspection and editing are strictly Plane 1 (`Engine-Maintenance`) only (FR-11).
//! - `[Claim-Hygiene-Grading]` All model provenance attestations must carry an explicit grade: A (locally proven), B (measured locally), C (inferred).

# Sovereign Blueprint: Weight-Space Resilience & Attestation Suite

**Slug**: `weight-space-resilience`  
**Target Milestone**: 1.1.465  
**Governing Skills**: `architecture`, `plan-writing`, `clean-code`  
**Execution Plane**: Plane 1 (Codebase Lifecycle Engine)

---

## 1. Executive Summary & Phased Architecture

Inspired by the MIT CSAIL *Weight-Space Programming* research (Takehi, Teoh, Zhang, Khattab, 2026), this suite introduces a 3-phase sovereign resilience and attestation framework that maps directly onto Tadpole OS's 3-Layer Architecture:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Operational Inference Resilience (Immediate)                  │
│  - Runner Action-Space Masking (loop_detector.rs)                     │
│  - Monologue Deduplication & Windowing (turn.rs)                       │
│  - 3-Shot Smoke Probe on local model pull (model_manager/)             │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 2: Model Provenance Attestation (P1)                             │
│  - Content-addressed SHA-256 weight digest & geometric fingerprinting │
│  - Policy SEC-09 in docs/SECURITY_REGISTRY.json                        │
│  - GET /v1/models/:id/attestation & Security Dashboard drift badge      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 3: Agent 99 Weight-Space Annealing (P2)                          │
│  - cap_type='weight_edit' in capability_proposals                      │
│  - Deterministic L3 projection in execution/weight_space/operators.py │
│  - Non-bypassable weights:edit oversight gate + Plane-1-only check     │
│  - Witness test harness with mandatory guardrails (no regression)      │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. The Sovereign 4-Pillar Breakdown

### Pillar 1: Gateway (Interface)

1. **Phase 1 Endpoints**:
   - `POST /v1/infra/models/{id}/probe`: Runs the bounded 3-shot micro-probe (schema, delimiter containment, loop resistance) against the local Ollama runtime.
   - WebSocket Event / Agent Pulse: `Attractor Break: Action-space mask active (excluded: [...])`.

2. **Phase 2 Endpoints**:
   - `GET /v1/infra/models/{id}/attestation`:
     ```json
     {
       "model_id": "qwen2.5:0.5b",
       "weight_digest": "sha256:7a8b9c...",
       "size_bytes": 397000000,
       "family": "qwen2",
       "parameter_size": "0.5B",
       "quantization_level": "Q4_K_M",
       "grade": "B",
       "baseline_digest": "sha256:7a8b9c...",
       "drift": false
     }
     ```
   - WebSocket Event: `engine:model_attestation_drift` emitted if disk digest diverges from registered baseline.

3. **Phase 3 Endpoints**:
   - Reuses existing proposal resolution endpoints:
     - `GET /v1/skills/proposals`: Lists proposals including `cap_type = 'weight_edit'`.
     - `POST /v1/skills/proposals/{id}/resolve`: Resolves proposal via HITL decision (`approved` / `rejected`).

4. **Frontend UI Integration**:
   - `src/pages/Model_Manager.tsx`: Displays provenance grade (`A`, `B`, `C`) and calibration status (`Calibrated` vs `Uncalibrated`).
   - `src/pages/Security_Dashboard.tsx`: Attestation trust badge and drift warning banner (`SEC-09`).

---

### Pillar 2: Runner (Logic)

1. **Action-Space Masking**:
   - Files: `server-rs/src/agent/runner/service_traits/orchestrator.rs` & `server-rs/src/agent/runner/intelligence/turn.rs`
   - In `RunContext`: `pub masked_tools: Arc<parking_lot::Mutex<HashSet<String>>>`.
   - On loop repetition warning, `orchestrator.rs` populates `active_ctx.masked_tools.lock().insert(tool_call.name.clone())`.
   - In `turn.rs`: `AgentRunner::apply_action_mask(active_ctx, &mut tool_def)` consumes and excludes masked tools for turn $N+1$, and restores them on turn $N+2$.

2. **Monologue Deduplication & Windowing**:
   - File: `server-rs/src/agent/runner/intelligence/monologue.rs`
   - `AgentRunner::prune_circular_monologue` measures 3-gram word Jaccard similarity across consecutive monologue turns with threshold `0.80` (and exact match for short thoughts >= 4 words).
   - Circular thoughts are replaced with `[Attractor Break: Circular reasoning detected and pruned. Formulate a new hypothesis or conclude.]`.

3. **3-Shot Micro-Probe Engine**:
   - File: `server-rs/src/routes/model_manager/probe.rs`
   - Runs Probe A (Schema JSON Extraction), Probe B (Delimiter Containment), and Probe C (Loop Resistance).
   - If Probe C fails, automatically clamp sampling parameters (`repeat_penalty: 1.15`, `temperature: 0.2`).

4. **Agent 99 Weight-Space Annealing**:
   - `Agent 99` authors proposals with `cap_type = 'weight_edit'`.
   - **Architectural separation**:
     - **L2 Orchestration (`Agent 99`)**: Identifies direction, layer index, and column scope.
     - **L3 Execution (`execution/weight_space/operators.py`)**: Executes deterministic arithmetic:
       $$W \leftarrow W - \hat{r}(\hat{r}^\top W)$$
     - **Witness Guardrails**: The intervention is accepted **only** if the primary failure metric improves *and* guardrail metrics (`tool_call_parse_success`, `instruction_following`) exhibit zero regression.

---

### Pillar 3: Registry (Data & State)

1. **Phase 1 In-Memory Registry**:
   - `AppState.resources` maintains `Arc<RwLock<HashMap<String, ModelProbeRecord>>>`.
   - Zero SQLite schema changes.

2. **Phase 2 Baseline Storage**:
   - Attestation baselines stored in existing `meta` / key-value table in SQLite (`knowledge_store_meta` or `system_settings`), matching registered model configs.

3. **Phase 3 Proposal Persistence**:
   - Reuses existing SQLite table `capability_proposals` (`id`, `name`, `description`, `cap_type`, `content`, `agent_id`, `status`).
   - Merkle audit logging via `execution/audit_chain.py`.

---

### Pillar 4: Security (Guard & Oversight)

1. **FR-11 Dual-Plane Sovereign Boundary**:
   - `weights:edit` and model attestation operations are strictly restricted to Plane 1 (`Engine-Maintenance`).
   - If an agent's plane is not Plane 1, the gate rejects fail-closed with HTTP 403.

2. **Non-Bypassable HITL Capability `weights:edit`**:
   - Added to `docs/SECURITY_REGISTRY.json` under `SEC-09` (Model Provenance & Weight Integrity).
   - Requires explicit human-in-the-loop oversight resolution via `submit_oversight`.

3. **Immutable Originals & Lineage**:
   - Never mutate canonical base models. Edits produce derived tagged models (e.g., `model:latest+tadpole-anneal-0001`) with the parent hash recorded in the Merkle audit trail.

---

## 3. Implementation Task Sequence & Agent Map

| Phase | Step | Task | Target Files | Primary Agent | Verification Gate |
| :---: | :---: | :--- | :--- | :---: | :--- |
| **P1** | **1.1** | **Action-Space Masking** | `server-rs/src/agent/runner/intelligence/turn.rs` | `rust-pro` | `cargo test test_action_space_masking_turn_filtering` |
| **P1** | **1.2** | **Monologue Circularity Pruning** | `server-rs/src/agent/runner/intelligence/monologue.rs` | `rust-pro` | `cargo test test_prune_circular_monologue` |
| **P1** | **1.3** | **3-Shot Micro-Probe Engine** | `server-rs/src/routes/model_manager/probe.rs` | `rust-pro` | `cargo test routes::model_manager::probe` |
| **P2** | **2.1** | **Deterministic Attestation Tool** | `execution/weight_space/attest.py` | `python-patterns` | `python execution/weight_space/attest.py --test` |
| **P2** | **2.2** | **Attestation Endpoint & SEC-09 Policy** | `server-rs/src/routes/model_manager/attestation.rs`, `docs/SECURITY_REGISTRY.json` | `rust-pro` | `GET /v1/infra/models/{id}/attestation` OpenAPI parity |
| **P3** | **3.1** | **Deterministic L3 Projection Operator** | `execution/weight_space/operators.py` | `python-patterns` | `python execution/weight_space/operators.py --test` |
| **P3** | **3.2** | **Weight-Edit Proposals & Oversight Gate** | `server-rs/src/agent/runner/mission_tools/lifecycle.rs`, `server-rs/src/routes/skills.rs` | `rust-pro` | `cargo test test_weight_edit_proposal` & `test_weight_edit_resolution` |
| **All** | **4.0** | **ADG v2 & Parity Verification** | Repository-wide | `adg-governance` | `npm run adg:verify` & `python execution/parity_guard.py` |

---

## 4. Definition of Done & Verification Mapping

1. **Witness Tests**:
   - `test_action_space_masking_turn_filtering`: Confirms turn $N+1$ tool declaration excludes the repeating tool and turn $N+2$ restores it.
   - `test_prune_circular_monologue`: Confirms circular monologue turn is pruned and diverse thought is preserved.
   - `test_evaluate_probe_response_healthy` / `test_evaluate_probe_response_looping`: Confirms 3-shot micro-probe scoring and parameter clamping.
   - `test_record_baseline_detects_drift`: Confirms SEC-09 in-memory attestation baseline and drift detection.
   - `test_weight_edit_proposal_persists_for_plane1_and_blocks_plane2`: Confirms FR-11 Plane 1 isolation and payload validation on capability proposals.
   - `test_weight_edit_resolution_records_decision_without_registering`: Confirms HITL approval path for `weight_edit` without creating rogue skill files.
   - `run_tests()` in `attest.py` and `operators.py`: Confirms deterministic SHA-256 digests, 64-element fingerprints, and mathematical column-scoped orthogonalization.
2. **Parity Gates**:
   - `npm run adg:verify` $\rightarrow$ 4/4 gates passing.
   - `npm run test:witness` $\rightarrow$ 0 drift violations.
   - `python execution/parity_guard.py` $\rightarrow$ 0 errors found.
