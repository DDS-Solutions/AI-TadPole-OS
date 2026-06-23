---
title: "Parity Guard"
tier: "3"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "none"
risk-tags:
  - "RISK: SAFE"
---

# 🛡️ Parity Guard

The `parity_guard.py` script serves as the codebase's **canonical drift detector**, ensuring documentation, Rust routes, environmental variables, and skill manifests remain in absolute alignment with the running code.

---

## 1. Invocation Details

### 1.1 Standard Verification Check

Run the verification check via the Python entry point:

```powershell
python execution/parity_guard.py .
```

- **Arguments**: First positional argument is the project root directory (`.` for current directory).
- **Exit Code 0**: Full alignment. No drift detected.
- **Exit Code 1**: Mismatches detected. Detailed failures printed to stdout.

### 1.2 Automated Fix Mode

To run the automated fix (such as synchronizing `API_REFERENCE.md` with modified OpenAPI schemas):

```powershell
python execution/parity_guard.py . FIX=1
```

- The `FIX=1` flag triggers `generate_api_reference.py` to regenerate `API_REFERENCE.md`.

---

## 2. Failure Modes Detected

The script evaluates and alerts on **five specific drift types**:

| # | Drift Type | Check | Severity |
|---|-----------|-------|----------|
| 1 | **Axum Route Drift** | Flags endpoints in `server-rs/src/router.rs` missing from `docs/openapi.yaml` | Error (`/v1` routes) / Warning (legacy routes) |
| 2 | **Environmental Drift** | Flags `std::env::var("VAR")` calls in Rust code not declared in `server-rs/.env.example` | Error |
| 3 | **Manifest Drift** | Validates `data/skills/*.json` manifests to ensure their designated execution scripts exist on disk | Error |
| 4 | **Documentation Drift** | Evaluates `@docs` tags in Rust code; if code changes occur, requires corresponding markdown file timestamp updates | Warning |
| 5 | **API Doc Drift** | Checks if `API_REFERENCE.md` timestamps lag behind `openapi.yaml` modifications | Error |

---

## 3. Exit Code Translation

| Exit Code | Meaning |
|-----------|---------|
| **0** | ✅ Full alignment. The system has no detected drift. |
| **1** | ❌ Mismatches detected. Detailed failures are printed to standard output. |

---

## 4. Output Format

The script prints results in a structured format:

```
[OK] [CHECK-NAME] Description of passing check
[FAIL] [CHECK-NAME] Description of failing check
```

**Example output:**
```
--- Tadpole OS Parity Audit ---

Found 42 routes in router.rs
[OK] [CODE->OPENAPI] GET /v1/agents
[FAIL] [CODE->OPENAPI] Route /v1/new-endpoint missing in openapi.yaml
[OK] [ENV-VAR] PORT documented
[FAIL] [ENV-VAR] std::env::var("NEW_VAR") used in code but missing from .env.example

Audit Complete. Errors found: 2
```

---

## 5. Scope & Limitations

> [!NOTE]
> **Parity Guard validates main-repo docs; wiki sync is a separate step.** The `parity_guard.py` script does not consume `OPERATIONS_MANUAL.md` or wiki pages directly. It runs independent AST scans against:
> - Rust source directories (`server-rs/src/`)
> - OpenAPI specification (`docs/openapi.yaml`)
> - Environment templates (`server-rs/.env.example`)
> - Skill manifests (`data/skills/*.json`)

### 5.1 What Parity Guard Does NOT Check

- Wiki page content or frontmatter
- Frontend component behavior
- Database migration state
- Runtime configuration values

---

## 6. Integration with CI/CD

Parity Guard is designed to run as a CI gate:

```yaml
# Example GitHub Actions step
- name: Run Parity Guard
  run: python execution/parity_guard.py .
```

A non-zero exit code will fail the CI pipeline, preventing drift from reaching production.

---

## 7. Interpreting Orphan Symbols

If Parity Guard flags a symbol as **orphan** (present in docs but not in code, or vice versa):

| Scenario | Action |
|----------|--------|
| Symbol exists in code but not docs | Add to `openapi.yaml` or `API_REFERENCE.md` |
| Symbol exists in docs but not code | Remove from docs, or mark `[INTERNAL]` if it's a planned feature |
| Environment variable used but undocumented | Add to `server-rs/.env.example` |
| Skill manifest references missing script | Create the script or remove the manifest |

→ *See [[Configuration|§1]] for environment variable reference.*
→ *See [[Developer-Appendix]] for the Rust routing architecture.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](../GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Parity-Guard)
