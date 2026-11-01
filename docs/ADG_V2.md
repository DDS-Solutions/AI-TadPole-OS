> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Sovereign Governance / Active Documentation Guard
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, hallucinated metadata, legacy v1 header anti-patterns.
> - **Observability**: Traceability via `tools/adg/adg.mjs` and `execution/verify_ai_context.py`

# Active Documentation Guard (ADG) v2 Specification

## 1. Executive Summary

**Active Documentation Guard (ADG) v2** is the sovereign metadata verification and integrity enforcement engine for Tadpole OS. It transitions the codebase from **unverified human/agent assertions** (ADG v1) to **deterministic, machine-verified contracts**.

In ADG v1, developers and AI agents manually typed fact claims into source file JSDoc blocks (e.g., `Witness Tests: ...`, `Invariants: ...`). Because these comments were not compiled or tested, they frequently drifted:
- Source files with robust tests claimed `"none declared"`.
- Files declared test paths that did not exist on disk.
- AI agents reading these headers treated them as absolute ground truth, leading to hallucinated context and brittle refactors.

**ADG v2 eliminates this failure mode** by strictly decoupling architectural guidance from machine-verifiable facts.

---

## 2. The 3-Layer Claim Taxonomy

ADG v2 classifies all repository claims into three distinct layers:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        ADG v2 3-Layer Claim Model                      │
├────────────────────────────────────────────────────────────────────────┤
│  P-Class (Prose & Guidance)       │ Source File JSDoc Headers         │
│  M-Class (Machine-Derived Facts)   │ adg.manifest.json                 │
│  E-Class (Executable Invariants)  │ tests/invariants/*.test.ts        │
└────────────────────────────────────────────────────────────────────────┘
```

### A. P-Class (Prose & Guidance)
- **Location**: Top-of-file JSDoc comments in source files (`src/**/*.ts(x)`).
- **Purpose**: High-level semantic context, architectural links (`@docs`), subsystem boundaries, and primary entrypoints.
- **Rule**: P-class comments describe *intent* and *structure*. They **MUST NOT** assert empirical facts about test coverage or security guarantees.
- **Forbidden in P-Class**:
  - `Witness Tests:`
  - `Invariants:` (unless purely descriptive architectural notes)
  - `Local Errors:`
  - `Test Coverage:`

### B. M-Class (Machine-Derived Facts)
- **Location**: [`adg.manifest.json`](file:///d:/TadpoleOS-Dev/adg.manifest.json).
- **Purpose**: Empirical repository facts computed deterministically by static analysis tools.
- **Contents**:
  - `claims.witness_map`: Deterministic co-location mapping of source files to test files.
  - `claims.csp`: Parsed Content Security Policy directives extracted from `tauri.conf.json`.
  - `verified_at`: Git commit hash / timestamp of the verification baseline.
- **Ratchet Rule**: The number of witnessed source files can never decrease without explicit administrative override.

### C. E-Class (Executable Invariants)
- **Location**: Dedicated test suites in [`tests/invariants/`](file:///d:/TadpoleOS-Dev/tests/invariants/).
- **Purpose**: Automated, executable verification of critical safety, crypto, security, and governance invariants.
- **Rule**: Every security or behavioral policy must be proven by a running test that fails if the code deviates.

---

## 3. Tooling & CLI Reference (`tools/adg/adg.mjs`)

ADG v2 provides a zero-dependency, high-speed CLI tool located at [`tools/adg/adg.mjs`](file:///d:/TadpoleOS-Dev/tools/adg/adg.mjs).

### Available Commands

| Command | npm Script | Description |
| :--- | :--- | :--- |
| `lint-headers <dir>` | `npm run adg:lint` | Scans source files and flags forbidden v1 header fields. |
| `codemod [--write] <dir>` | `npm run adg:codemod` | Strips forbidden v1 header fields while preserving P-class architectural guidance. |
| `generate [--check] <dir>` | `npm run adg:generate` / `npm run adg:check` | Computes witness maps, parses CSP, checks ratchets, and writes or validates `adg.manifest.json`. |
| `verify <dir>` | `npm run adg:verify` | Complete verification pipeline: runs must-fail fixtures, lint-headers, CSP grammar, and manifest integrity. |

### CI Verification Scripts

In [`package.json`](file:///d:/TadpoleOS-Dev/package.json), the standard CI hooks are wired directly to ADG v2:
```json
"verify:adg": "node tools/adg/adg.mjs verify src/",
"test:witness": "node tools/adg/adg.mjs verify src/"
```

---

## 4. Must-Fail Meta-Test Fixtures

To ensure the verification tool itself never suffers from false negatives, ADG v2 includes positive proof fixtures in [`tools/adg/fixtures/must-fail/`](file:///d:/TadpoleOS-Dev/tools/adg/fixtures/must-fail/):

1. **`v1_header.ts`**: Contains a forbidden `Witness Tests:` header. Proves `lint-headers` catches legacy metadata.
2. **`fabricated_witness.ts`**: Declares a nonexistent witness test. Proves phantom test references cannot slip through.

`node tools/adg/adg.mjs verify src/` automatically tests these fixtures on every execution.

---

## 5. Seed Executable Invariants Catalog

The initial seed invariant test suites in [`tests/invariants/`](file:///d:/TadpoleOS-Dev/tests/invariants/) assert repository-wide behavioral contracts:

1. **`INV-001` ([`inv_001_oversight.test.ts`](file:///d:/TadpoleOS-Dev/tests/invariants/inv_001_oversight.test.ts))**: User question decisions strictly map to `'approved'` vs `'rejected'`, match on exact `question_id`, and revert on network failure.
2. **`INV-002` ([`inv_002_vault_pbkdf2.test.ts`](file:///d:/TadpoleOS-Dev/tests/invariants/inv_002_vault_pbkdf2.test.ts))**: Vault master key derivation enforces $\ge 600,000$ PBKDF2 iterations, produces non-extractable keys, and verifies AES-GCM encryption.
3. **`INV-003` ([`inv_003_telemetry_scrub.test.ts`](file:///d:/TadpoleOS-Dev/tests/invariants/inv_003_telemetry_scrub.test.ts))**: Telemetry redacts Google, Anthropic, OpenAI, Groq, HuggingFace, and GitHub API keys, Bearer tokens, and nested object fields.
4. **`INV-004` ([`inv_004_ws_cooldown.test.ts`](file:///d:/TadpoleOS-Dev/tests/invariants/inv_004_ws_cooldown.test.ts))**: WebSocket reconnection exponential backoff enforces a 90s cooldown timer and online event recovery.
5. **`INV-005` ([`inv_005_openui_depth.test.ts`](file:///d:/TadpoleOS-Dev/tests/invariants/inv_005_openui_depth.test.ts))**: OpenUI JSON rendering enforces `MAX_RENDER_DEPTH \le 10` with fallback component badge.
6. **`INV-006` ([`inv_006_csp_grammar_meta.test.ts`](file:///d:/TadpoleOS-Dev/tests/invariants/inv_006_csp_grammar_meta.test.ts))**: Meta-test verifying ADG parser catches RFC 1918 wildcards, unquoted keywords, and unsafe scripts.
7. **`INV-SEC-001` ([`inv_sec_001_settings_persist.test.ts`](file:///d:/TadpoleOS-Dev/tests/invariants/inv_sec_001_settings_persist.test.ts))**: Settings store partialize blanks API keys, migrates from localStorage, and purges legacy tokens.
8. **`INV-SEC-002` ([`inv_sec_002_session_storage.test.ts`](file:///d:/TadpoleOS-Dev/tests/invariants/inv_sec_002_session_storage.test.ts))**: Strict AST/static scan guaranteeing zero writes or reads of auth tokens to/from `sessionStorage`.

---

## 6. Integration with the Python AI Gate

[`execution/verify_ai_context.py`](file:///d:/TadpoleOS-Dev/execution/verify_ai_context.py) integrates seamlessly with ADG v2:
- Gates 1–4 continue to verify header presence, documentation links, telemetry tags, and AST symbols.
- Gate 6 queries [`adg.manifest.json`](file:///d:/TadpoleOS-Dev/adg.manifest.json) to resolve witness claims for any source file that has migrated to ADG v2 (no manual headers needed).
- All 617 source files are verified continuously across both TypeScript and Python pipelines.

---

## 7. Agent Guidelines for New Code

When authoring or modifying code in this codebase:
1. **Header Format**: Include `@docs`, `### AI Context Alignment` (subsystem & entrypoints), and `### 🔍 Debugging & Observability`.
2. **Never Hand-Write Witness Tests**: Do NOT write `Witness Tests:` or `Invariants:` in the JSDoc header.
3. **Co-Locate Tests**: Place your unit test adjacent to the implementation (e.g., `my_service.ts` $\rightarrow$ `my_service.test.ts`).
4. **Regenerate Manifest**: Run `npm run adg:generate` after adding new tests to update `adg.manifest.json`.
5. **Verify**: Ensure `npm run adg:verify` and `npx vitest run tests/invariants/` both pass.
