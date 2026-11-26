---
title: "Parity Guard"
tier: "3"
status: "verified"
version: "1.1.464"
last-verified: "2026-11-20"
commit: "3eef5c2d"
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

The script executes **12 automated verification checks** (`execution/parity_guard.py` + `execution/lib/parity_validators.py`):

| # | Check | Description |
|---|-------|-------------|
| 1 | **Axum Route → OpenAPI** | Flags `/v1` & `/a2a` routes in `server-rs/src/router.rs` missing from `docs/openapi.yaml` |
| 2 | **`@docs` Tag Drift** | Evaluates `@docs` tags in Rust code against documentation timestamps |
| 3 | **Environment Var Parity** | Flags `std::env::var("VAR")` calls in Rust code missing from `.env.example` |
| 4 | **Skill Manifest Parity** | Validates `data/skills/*.json` manifests to ensure execution scripts exist on disk |
| 5 | **API Reference Parity** | Verifies `docs/API_REFERENCE.md` is up to date with `docs/openapi.yaml` and route annotations |
| 6 | **Pollywog Debt Audit** | Checks `POLLYWOG-DEBT.md` debt ledger compliance |
| 7 | **Critical Docs Content** | Validates required governance sections in core root/docs files |
| 8 | **Wiki Metadata Parity** | Validates `docs/wiki/` page frontmatter and version metadata (`_check_wiki_metadata`) |
| 9 | **Backtest Engine** | Validates deterministic benchmark/backtest contracts |
| 10 | **Security Registry Parity** | Verifies `docs/SECURITY_REGISTRY.json` alignment |
| 11 | **Documentation Link Guard** | Runs `execution/verify_doc_links.py` across `docs/`, `.agent/`, and root guides |
| 12 | **Dual-Plane Identity & Rust Test Compilation** | Verifies `FR-11` / Dual-Plane markers & `IDENTITY.md` version parity across all Markdown files |

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
> **Parity Guard validates structural & metadata parity; prose accuracy still requires periodic semantic audits.** `parity_guard.py` checks Rust routes, `.env.example`, skill manifests, `docs/openapi.yaml`, `docs/SECURITY_REGISTRY.json`, `docs/wiki/` frontmatter metadata, doc links, and Dual-Plane `IDENTITY.md` markers. Wiki publishing to the public `AI-TadPole-OS` Wiki is handled separately by `.github/workflows/wiki-sync.yml`.

### 5.1 What Parity Guard Does NOT Check

- Semantic accuracy of freeform prose explanations inside docs or wiki pages
- Frontend UI runtime behavior (covered by Vitest & Playwright)
- Live database migration state or runtime environment values

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
| Environment variable used but undocumented | Add to `.env.example` |
| Skill manifest references missing script | Create the script or remove the manifest |

→ *See [[Configuration]] for environment variable reference.*
→ *See [[Developer-Appendix]] for the Rust routing architecture.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Parity-Guard)


[//]: # (Metadata: [wiki-security])
