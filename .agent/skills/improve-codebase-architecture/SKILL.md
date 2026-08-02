---
name: improve-codebase-architecture
description: Scans git commit hot spots for shallow modules, generates visual HTML reports with Mermaid diagrams, and drills into refactoring candidates.
when_to_use: "Use when evaluating codebase friction, refactoring shallow modules, or generating visual architecture reviews."
allowed-tools: Read, Glob, Grep, Bash, Write
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for scanning architectural friction and generating visual HTML refactoring reports.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

# Codebase Architecture Improvement Protocol

> Source: mattpocock/skills (improve-codebase-architecture)

Scan the codebase for architectural friction and propose **deepening opportunities**—refactors that transform shallow modules into deep ones.

---

## 1. Deep Module Vocabulary

Every proposal must strictly use deep module design terms:
- **Module**: Scale-agnostic unit with an interface and implementation (function, struct, package, tier slice).
- **Interface**: Everything a caller must know (type signature, invariants, error modes, performance bounds).
- **Depth**: Ratio of implementation capability to interface complexity. (Deep = high capability behind small surface).
- **Seam**: Location where behavior can be altered without editing callers.
- **Adapter**: Concrete implementation filling a seam slot.
- **Leverage**: Capability callers gain per unit of interface learned.
- **Locality**: Concentration of changes, bugs, and verification in one place.

---

## 2. Exploration Phase

1. **Scan Commit Hot Spots**: Run `git log --oneline -n 30` to identify high-churn modules.
2. **Apply Deletion Test**: If you delete a module, does it concentrate complexity (deep module), or merely scatter it (shallow module)?
3. **Identify Friction**:
   - Shallow interfaces almost as complex as implementations.
   - Pure functions extracted for testing where real bugs hide in caller wiring (poor locality).
   - Tightly-coupled modules leaking across seams.

---

## 3. Generate Visual HTML Report

Write an offline HTML report to `.tmp/reports/architecture-review-<timestamp>.html`.

### HTML Report Requirements
- **CDN Styling**: Tailwind CSS CDN + Mermaid CDN.
- **Visual Cards**: Render cards for each refactoring candidate:
  - **Modules/Files Involved**: Relative file paths.
  - **Problem Statement**: Architecture friction described in terms of locality & depth.
  - **Proposed Solution**: Clear refactoring description.
  - **Before / After Diagrams**: Mermaid sequence/flowcharts showing structural deepening.
  - **Recommendation Badge**: `Strong`, `Worth Exploring`, `Speculative`.
- **Top Recommendation**: Clear summary of the highest-leverage candidate.

Open the file automatically (`start <path>` on Windows, `open <path>` on macOS, `xdg-open <path>` on Linux) and display the file path.
