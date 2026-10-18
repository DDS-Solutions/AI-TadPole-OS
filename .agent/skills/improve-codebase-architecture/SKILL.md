---
name: improve-codebase-architecture
description: "Scans git commit hot spots for shallow modules, generates visual HTML reports with Mermaid diagrams, and drills into refactoring candidates."
when_to_use: "When evaluating codebase friction, refactoring shallow modules, auditing blast radius, or generating visual architecture reviews."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / improve-codebase-architecture
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Shallow module proliferation, scattered locality, or leaky abstraction seams.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Codebase Architecture Improvement Protocol

> **Purpose**: Scan the codebase for architectural friction, identify shallow modules and high-churn hotspots, compute symbol blast radius via the Sovereign Symbol Graph, and propose **deepening opportunities** that transform shallow pass-through layers into robust, high-leverage engines.  
> **Workflow Bindings**: Integrated into [`/refactor`](file:///D:/TadpoleOS-Dev/.agent/workflows/refactor.md) and [`/audit`](file:///D:/TadpoleOS-Dev/.agent/workflows/audit.md).

---

## 📚 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/deep_module_design_patterns.md`](file:///D:/TadpoleOS-Dev/.agent/skills/improve-codebase-architecture/references/deep_module_design_patterns.md) | Ousterhout's Deep Module theory, the 7 design concepts, and refactoring patterns | Planning refactors, evaluating module interfaces, or decomposing hubs |
| [`scripts/architecture_scanner.py`](file:///D:/TadpoleOS-Dev/.agent/skills/improve-codebase-architecture/scripts/architecture_scanner.py) | Automated git hotspot scanner and offline visual HTML report generator | Initiating an architecture audit or locating high-churn friction |
| [`code-review-graph`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-graph/SKILL.md) | Symbol graph call-tree inspector and blast radius calculator (`npm run graph:blast`) | Measuring blast radius before editing Rust hubs or React stores |

---

## 🏛️ 1. The 7 Deep Module Vocabulary Concepts

Every architectural review and refactoring proposal must strictly employ the **Deep Module Vocabulary**:

1. **Module**: A scale-agnostic unit with an interface and implementation (function, struct, actor, hub, store, package).
2. **Interface**: Everything a caller must know (type signatures, preconditions, error modes, side-effects, invariants).
3. **Depth**: The ratio of implementation capability to interface complexity. High capability behind a narrow surface.
4. **Seam**: An isolation point where behavior can be varied or substituted without editing callers.
5. **Adapter**: A concrete implementation plugging into a seam (e.g. LLM providers).
6. **Leverage**: The capability callers gain per unit of interface complexity learned.
7. **Locality**: Concentration of related logic, changes, bug fixes, and verification in a single place.

---

## 🔄 2. Operational Architecture Audit Workflow

```
[1. SCAN HOTSPOTS & DEPTH] ───> [2. BLAST RADIUS] ───> [3. VISUAL REPORT] ───> [4. DEEPENING ADR]
  Run architecture_scanner        Run graph:blast       Generate HTML cards       Submit refactor plan
```

1. **Step 1: Scan Hotspots & Module Depth**
   Execute the automated scanner to pinpoint high-churn files and shallow interfaces:
   ```powershell
   python .agent/skills/improve-codebase-architecture/scripts/architecture_scanner.py src --hotspots 30
   ```
2. **Step 2: Inspect Symbol Blast Radius**
   Use the Sovereign Symbol Graph to inspect callers and callees of candidate modules:
   ```powershell
   npm run graph:blast:guard -- --path server-rs/src/routes/intelligence.rs
   ```
3. **Step 3: Review Visual HTML Report**
   Inspect the interactive HTML report generated at `.tmp/reports/architecture-review-<timestamp>.html` featuring embedded Mermaid sequence flows and recommendation badges (`Strong`, `Worth Exploring`, `Speculative`).
4. **Step 4: Formulate Deepening Refactoring Contract**
   Draft a clear refactor proposal specifying:
   - The shallow interface being collapsed.
   - The hidden complexity (caching, concurrency, retry logic) being pulled into the deep module.
   - The reduction in caller lines of code and cognitive overhead.

---

## 🧠 3. Socratic Gate for Architectural Deepening

Before proposing or executing an architectural refactor, answer these 3 gates:
- **Gate 1 (Leverage & Depth)**: Does the proposed new module interface hide significant complexity, or does it merely pass arguments through to another layer?
- **Gate 2 (Locality & Blast Radius)**: Does this refactor consolidate scattered changes into a single cohesive module, or does it scatter responsibility across more files?
- **Gate 3 (Seam Integrity)**: Does the module provide clear seams (via traits or interfaces) so that concrete adapters can be swapped without modifying call sites?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Classitis / Pass-Throughs** | Creating dozens of 20-line classes/functions that only forward calls. | Collapse shallow layers into a unified deep engine. |
| **Temporal Decomposition** | Splitting logic into Step1, Step2, Step3 modules rather than by information hiding. | Reorganize modules around shared state and data structures. |
| **Scattered Locality** | Making a feature change requires touching 7 different files in different directories. | Pull callers' coordination logic into the domain engine. |
| **Leaky Seams** | Callers must know implementation details (e.g. SQLite specifics or raw HTTP headers). | Hide storage and transport details entirely behind domain interfaces. |
| **Unbounded God Objects** | Modules $> 500$ LOC with $> 15$ public exports and immense blast radius. | Extract self-contained cohesive sub-engines and narrow the export surface. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate the architecture scanner tool and run codebase audits:

```powershell
# 1. Run built-in self tests
python .agent/skills/improve-codebase-architecture/scripts/architecture_scanner.py --test

# 2. Audit codebase and generate visual report
python .agent/skills/improve-codebase-architecture/scripts/architecture_scanner.py src --hotspots 30
```