---
name: documentation-templates
description: "Standardized documentation protocols for humans and AI. Manages READMEs, API specs, ADRs, and AI-native context maps."
when_to_use: "When creating or updating project documentation, writing Architecture Decision Records (ADRs), or generating AI context maps."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / documentation-templates
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Undocumented architectural drift, non-standard ADR structures, broken AI context headers.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Documentation Protocols & Knowledge Heritage Standards

> **Philosophy**: Documentation is Infrastructure. It eliminates decision regressions, preserves institutional knowledge, and accelerates autonomous agent reasoning.
> **Workflow Binding**: Used directly during [`/codify`](file:///D:/TadpoleOS-Dev/.agent/workflows/codify.md), [`/report`](file:///D:/TadpoleOS-Dev/.agent/workflows/report.md), and [`/audit`](file:///D:/TadpoleOS-Dev/.agent/workflows/audit.md).

---

## 📚 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Resource | Scope & Purpose | Trigger / When to Load |
|---|---|---|
| [`adr_and_llms_templates.md`](file:///D:/TadpoleOS-Dev/.agent/skills/documentation-templates/references/adr_and_llms_templates.md) | Standard ADR blueprint, `llms.txt` schema, ADG v2 multi-language AI context headers | Creating ADRs, generating context catalogs, authoring code headers |
| [`doc_template_linter.py`](file:///D:/TadpoleOS-Dev/.agent/skills/documentation-templates/scripts/doc_template_linter.py) | Deterministic markdown & ADR template structural validator | Pre-commit verification of docs and decision records |
| [`/codify` Workflow](file:///D:/TadpoleOS-Dev/.agent/workflows/codify.md) | Knowledge heritage synthesis pipeline | Translating verified patterns into sovereign directives |

---

## 🏛️ 1. The 4-Tier Documentation Taxonomy

```
Tier 1: Architectural Directives  ➔ docs/ & docs/adr/ (Sovereign invariants, system boundaries)
Tier 2: Inline Context Contracts   ➔ Code Headers & Docstrings (Enforced via verify_ai_context.py & ADG v2)
Tier 3: Machine Contracts          ➔ docs/openapi.yaml & docs/API_CONTRACT.md (Typed interfaces)
Tier 4: Knowledge Heritage         ➔ wiki/ & docs/wiki/ (Operational guides, runbooks, debriefs)
```

---

## 🔄 2. The 5-Phase Knowledge Lifecycle

```
1. BRAINSTORM ➔ (/brainstorm) Divergent architectural exploration and trade-off evaluation.
2. DECIDE     ➔ Formalize architectural contract into an ADR under `docs/adr/`.
3. IMPLEMENT  ➔ Execute code changes with aligned AI Context headers.
4. VERIFY     ➔ Verify parity with `parity_guard.py` and run executable test suites.
5. CODIFY     ➔ (/codify) Crystallize learnings into permanent directives and wiki runbooks.
```

---

## ✍️ 3. The Sovereign Commenting Standard ("Comment the Why, Not the What")

- **MUST Comment**:
  - Non-obvious architectural trade-offs ("Using B-tree instead of Hash due to range scan query patterns").
  - Safety & concurrency hazards ("Never invoke this function while holding the state lock").
  - Algorithmic complexity bounds ($O(N \log N)$) and memory footprint limits.
  - Links to external specifications, RFCs, or ADR numbers.
- **NEVER Comment**:
  - Trivial code restatements (`// increment counter`).
  - Duplicated type annotations or redundant function signatures.
  - Commented-out dead code (delete it; git preserves history).

---

## 🛡️ 4. Active Documentation Guard (ADG v2) Invariants

> 🔴 **No Hand-Written Fact Claims**: Never write `Witness Tests:` or `Invariants:` inside source file headers under `src/`. Those are forbidden v1 anti-patterns flagged by `npm run adg:lint`.

1. **Manifest Integrity**: Test witness relationships are derived automatically by `npm run adg:generate` into `adg.manifest.json`.
2. **Executable Invariants**: Safety guarantees must be executable test suites residing in `tests/invariants/` (verified via `npx vitest run tests/invariants/`).
3. **Docs Link Parity**: All `@docs` tags in headers must correspond to real registered subsystem anchors in `docs/ARCHITECTURE.md`.

---

## 🚫 5. Critical Documentation Anti-Patterns

- ❌ **Unrecorded Architectural Pivots**: Changing database engines, state stores, or protocols without filing an ADR in `docs/adr/`.
- ❌ **Broken Relative Links**: Using relative directory jumps (`../../`) that break when documents move. Always use GitHub-style `file:///` links.
- ❌ **Documentation Drift**: Altering API responses or environment variables without running `python execution/parity_guard.py` to synchronize docs.
- ❌ **Stale Comment Rot**: Leaving comments that contradict updated code logic.

---

## 🛠️ 6. Verification Commands

Run deterministic documentation linting and parity verification:

```powershell
# 1. Run doc template linter self-tests
python .agent/skills/documentation-templates/scripts/doc_template_linter.py --test

# 2. Audit all documentation files in docs/
python .agent/skills/documentation-templates/scripts/doc_template_linter.py docs

# 3. Verify Active Documentation Guard (ADG v2)
npm run adg:verify

# 4. Run documentation parity guard
python execution/parity_guard.py
```