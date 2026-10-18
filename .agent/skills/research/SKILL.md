---
name: research
description: "Executes isolated research and investigation subagents on throwaway git branches to prevent cluttering main working state."
when_to_use: "Use when investigating third-party APIs, reading external docs, benchmarking libraries, or building throwaway research spikes."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: MEDIUM
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / research
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Polluting main branch with exploratory throwaway code or failing to distill findings into actionable dossiers.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Isolated Research Subagent Protocol

> **Purpose**: Execute technical investigations, external API audits, and library benchmarks on isolated throwaway branches or within `.tmp/` to guarantee zero dirty state in main working trees.  
> **Workflow Bindings**: Integrated into [`/brainstorm`](file:///D:/TadpoleOS-Dev/.agent/workflows/brainstorm.md), [`/plan`](file:///D:/TadpoleOS-Dev/.agent/workflows/plan.md), and [`/github_scout`](file:///D:/TadpoleOS-Dev/.agent/workflows/github_scout.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/research_dossier_templates.md`](file:///D:/TadpoleOS-Dev/.agent/skills/research/references/research_dossier_templates.md) | Standard dossier schema, comparison matrix layout, and gotcha logs | Formatting findings in `.tmp/research-<topic>.md` |
| [`scripts/research_guard.py`](file:///D:/TadpoleOS-Dev/.agent/skills/research/scripts/research_guard.py) | Automated research dossier linter checking required sections and actionable recommendations | Validating research summaries before presentation |

---

## 🔬 1. The 4-Phase Research Protocol

```
[1. BRANCH/TMP] ───> [2. INVESTIGATE] ───> [3. DOSSIER] ───> [4. RESTORE]
 Create throwaway     Query APIs, read     Write summary into Return to main
 branch or .tmp/      docs, test spikes    .tmp/research-*.md branch & cleanup
```

1. **Create Throwaway Isolation**: When testing heavy code, create a scratch branch: `git checkout -b research/<topic>`. For documentation/API queries, work directly inside `.tmp/`.
2. **Execute Research Subagent**: Spawn dedicated research subagent to query documentation, test API payloads, or benchmark algorithms.
3. **Capture Findings**: Formulate a structured dossier in `.tmp/research-<topic>.md` containing Executive Summary, Comparison Matrix, and Actionable Next Steps.
4. **Clean Up Working State**: Return to original branch (`git checkout -`) and prune throwaway branches (`git branch -D research/<topic>`).

---

## 🧠 2. Socratic Gate for Research Tasks

Before beginning an exploratory research spike, verify:
- **Gate 1 (Isolation Bound)**: Are all code experiments confined to a throwaway branch or `.tmp/`?
- **Gate 2 (Decision Target)**: What specific architectural question will this research answer?
- **Gate 3 (Dossier Delivery)**: Will the findings be distilled into `.tmp/research-<topic>.md` rather than lost in chat history?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Dirty Working Tree** | Running experimental npm installs or file changes directly on `main`. | Checkout throwaway branch or run within `.tmp/`. |
| **Undocumented Findings** | Conducting deep exploration but leaving no permanent artifact. | Record structured dossier in `.tmp/research-<topic>.md`. |
| **Lingering Scratch Branches** | Accumulating dozens of dead `research/*` branches. | Prune branch immediately after findings are codified. |

---

## 🛠️ 4. Deterministic Verification Gate

Validate research dossier format and completeness:

```powershell
# 1. Run built-in self-tests
python .agent/skills/research/scripts/research_guard.py --test

# 2. Lint a research dossier
python .agent/skills/research/scripts/research_guard.py --file .tmp/research-vector-db.md
```