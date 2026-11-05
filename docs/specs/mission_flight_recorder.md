> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: UI Pages & Observability / Mission Flight Recorder
> - **Architecture**: `@docs ARCHITECTURE:Interface`
> - **Failure Path**: Disconnected telemetry, unverified execution claims, or fragmentation between active clusters and historical runs.
> - **Observability**: Live and historical mission metrics, Merkle audit chains, CAS diffs, and TrustGraph resolution.

# Feature Specification: Mission Flight Recorder & Run History

- **Status**: Implemented & Verified in Production Pipeline.
- **Decision Date**: 2026-10-26
- **Selected Approach**: Option B (Dedicated Master-Detail Flight Recorder in `/missions` with cross-link from `/benchmarks`).
- **Design Standard**: Strictly adheres to [`DESIGN.md`](file:///d:/TadpoleOS-Dev/DESIGN.md) (Deep HSL Tinted Obsidian `#090d16`, glassmorphic surfaces, JetBrains Mono metrics, and high-contrast telemetry tokens).

---

## 1. Overview & Objective

During autonomous execution, the Tadpole OS Sovereign Engine generates rich, multi-dimensional telemetry across relational records (`mission_history`, `mission_logs`, `swarm_context`), Content-Addressed Storage (`file_revisions`, `cas_blobs`), dense vector memory (`LanceDB`), Merkle audit trails, A2A economic settlement ledgers, and Agent 99 drift debriefs.

This specification documents the **Mission Flight Recorder & Run History** dashboard:
1. Adds a high-density, first-class historical dossier view inside the **Missions Hub** (`/missions`).
2. Provides an interactive view toggle between `Active Operations` and `Flight Recorder & Run History`.
3. Integrates an operational cross-link on the `/benchmarks` header card (`BenchmarkHeaderCard.tsx`) to provide instant navigation from analytics without conflating synthetic latency stress tests with business workflow history.
4. Implements cryptographic Merkle chain validation, PathGuard directory traversal defenses, and truthful telemetry representations.

---

## 2. Ubiquitous Domain Terminology

- **Mission Run**: A discrete, tracked execution lifecycle instance identified by a unique UUID (`msn-...` or custom cluster ID) mapped to `mission_history`.
- **Flight Dossier**: The comprehensive 6-pane forensic inspection view for a selected mission run.
- **Merkle Audit Trace**: Monotonically linked execution log steps where each turn contains `hash = SHA256(prev_hash + mission_id + agent_id + source + text + timestamp)`.
- **Pre-Mutation Snapshot (CAS)**: SHA-256 hash and binary payload captured in Content-Addressed Storage prior to an agent file write.
- **Memory Palace**: Spatial semantic groupings (`Rooms`) of swarm findings persisted under `swarm_context`.
- **Behavioral Drift Score**: Normalized cosine distance between an agent's runtime action vector and core identity embedding; returns `null` (`UNPROFILED`) when vector memory is unprofiled.
- **Two-Phase Commit (2PC) Lock**: An atomic escrow lock (`transaction_locks`) securing micro-USDC funds between agents during an A2A capability exchange.

---

## 3. Functional Requirements

- **FR-1**: Persistent view-mode toggle in `/missions` header allowing operators to switch seamlessly between `Active Operations` (live staging canvas) and `Flight Recorder & Run History` (historical dossier).
- **FR-2**: Operational shortcut link in [`BenchmarkHeaderCard.tsx`](file:///d:/TadpoleOS-Dev/src/components/benchmark/BenchmarkHeaderCard.tsx) navigating directly to `/missions?view=history`.
- **FR-3**: Filterable **Run Catalog** (Master list) supporting filtering by status (`All`, `Done`, `Failed`, `Active`, `Paused`), search query, and pagination (25 items/page).
- **FR-4**: Execution badges on each catalog item: Status Pill, Mission Title, Created Date, Log Turn Count, CAS Diff Count, and Cost in USD.
- **FR-5**: **Mission Flight Dossier** detail pane displaying 6 specialized forensic tabs:
  - **Tab 1: Telemetry & Metrics Rollup**: Total turns, error density, warnings, CAS mutation count, token counts, and cost breakdown.
  - **Tab 2: Merkle Audit Trace**: Chronological step logs with cryptographic SHA-256 verification badges, source agents, pruned payloads, and RFC-3339 timestamps.
  - **Tab 3: CAS File Diff Inspector**: Content-Addressed pre-mutation BLOB comparison against disk state with line addition/deletion summaries.
  - **Tab 4: Memory Palace (Swarm Context)**: Spatial findings grouped by topic room with agent attribution.
  - **Tab 5: TrustGraph & Economics**: Directed relation graph (`MISSION → OUTCOME → FILE`) and A2A transaction ledger.
  - **Tab 6: Agent 99 Debrief**: Behavioral drift score, mission verdict, and operational recommendations.
- **FR-6**: Deep-linking support via URL search params (e.g., `/missions?view=history&id=<mission_id>`).
- **FR-7**: Retention pin toggle to protect critical missions from automatic Reaper pruning cycles.

---

## 4. Security & Cryptographic Guarantees

- **SEC-01 (PathGuard Traversal Defense)**: All CAS file diff inspections pass through `crate::security::path_guard::validate_path` before reading disk content.
- **SEC-02 (File Size Ceiling)**: Files exceeding 1 MB are capped at metadata inspection to protect engine memory from heap exhaustion.
- **SEC-03 (Strict Merkle Audit)**: Hash validation uses shared `compute_log_hash` and canonical RFC-3339 timestamps; tamper anomalies immediately emit `tracing::warn!` and display high-contrast danger indicators.
- **SEC-04 (SQL Injection Immunity)**: Dynamic catalog filtering employs `sqlx::QueryBuilder::<Sqlite>` with parameter binding and escaped wildcards (`%`, `_`).
- **SEC-05 (Truthful Telemetry)**: Unmeasured metrics are represented truthfully as `null` / `UNPROFILED` rather than synthetic mock data.

---

## 5. System Contracts & Interfaces

### 5.1 REST API Contracts

```typescript
// GET /v1/missions?page=1&per_page=25&status=all&search=
export interface CatalogResponse {
    success: boolean;
    total: number;
    page: number;
    per_page: number;
    data: MissionSummaryItem[];
}

export interface MissionSummaryItem {
    id: string;
    agent_id: string;
    title: string;
    status: string;
    budget_usd: number;
    cost_usd: number;
    is_pinned: boolean;
    created_at: string;
    updated_at: string;
    log_count: number;
    diff_count: number;
}

// GET /v1/missions/:id/dossier
export interface MissionDossier {
    mission: MissionSummaryItem;
    logs: MerkleStepRecord[];
    diffs: CasDiffRecord[];
    memory_palace: SwarmFindingRecord[];
    trust_graph: TrustGraphPayload;
    economics: EconomicsPayload;
    debrief: DebriefPayload;
}

// POST /v1/missions/:id/pin
export interface PinResponse {
    success: boolean;
    is_pinned: boolean;
    mission_id: string;
}
```

### 5.2 Component Hierarchy & Seams

```text
src/pages/Missions.tsx
 ├── Header (View Mode Toggle: Active Canvas <-> Flight Recorder)
 └── [view === 'history'] -> Mission_History_View
      ├── Mission_Catalog_Sidebar (Search, Status Filter, Pagination, Card List, Error Banner)
      └── Mission_Flight_Dossier
           ├── Dossier Header (Status, ID, Agent, Timestamps, Pin Toggle)
           ├── Metrics_Rollup_Pane (KPI Cards, Drift, Cost, Turns)
           ├── Navigation Tabs (6 Panes)
           │    ├── Merkle_Audit_Viewer
           │    ├── CAS_Diff_Inspector
           │    ├── Memory_Palace_Viewer
           │    ├── TrustGraph_Economics_Viewer
           │    └── Agent99_Debrief_Viewer
           └── Error & Loading States (Retry Sync & Reload Dossier)
```

---

## 6. Verification Evidence

- **Frontend Build**: `npx tsc -b` &rarr; 0 errors.
- **Backend Build**: `cargo check --manifest-path server-rs/Cargo.toml --bin server-rs` &rarr; 0 errors, 0 warnings.
- **Service Test Suite**: `npx vitest run src/services/mission_history_service.test.ts` &rarr; 4 passed.
- **Merkle Chain Tests**: `cargo test --bin server-rs agent::mission::tests` &rarr; 3 passed.
- **ADG v2 Manifest & Lint**: `npm run adg:verify` &rarr; 652 files scanned, 0 violations.
- **Browser Validation**: Fully verified in Chromium via browser subagent.
