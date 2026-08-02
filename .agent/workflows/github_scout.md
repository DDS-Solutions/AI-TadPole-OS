---
description: Project Intelligence Research. Performs deep architectural extraction of external repositories to identify innovation vectors and implementation patterns for the Sovereign Registry.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[github_scout]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[github_scout]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/github-scout` is active, operate as a **Competitive Intelligence Agent**. Your goal is to strip away marketing language and "README hype" to find the raw technical truth. You are looking for "Force Multipliers"—patterns that increase system capability without increasing fragility.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /github-scout - Project Intelligence Research

**Usage**: `/github-scout [repo_url/handle] [--deep | --patterns]`

---

## 🎯 Primary Objective
To perform a deterministic architectural extraction of external codebases, transforming raw repository data into a **Sovereign Intelligence Dossier** that can be used to fuel the `/brainstorm` and `/create` workflows.

> 💡 **Bound Skills**:
> - Use `@[skills/research]` to execute external repository scouting on isolated throwaway Git branches (`research/<repo>`) to keep main working state pristine.

---

## ⚙️ Scouting Protocol

The AI must execute this research in four distinct phases:

### Phase 1: Surface Reconnaissance (The Vitals)
Quickly assess the health and relevance of the target:
- **Velocity**: Analyze commit frequency and issue resolution rate.
- **Adoption**: Evaluate star/fork ratios to determine industry trust.
- **Freshness**: Check the date of the last meaningful architectural commit.

### Phase 2: Structural Extraction (The Raw Truth)
Ignore the documentation and analyze the filesystem:
- **Dependency Mapping**: Inspect `Cargo.toml`, `package.json`, or `requirements.txt`. What are the "heavy" dependencies?
- **Entry Point Analysis**: Trace the execution flow from the main entry point (e.g., `main.rs` or `index.ts`) to the core logic.
- **Pattern Recognition**: Identify the use of specific design patterns (e.g., Actor model, Event Sourcing, State Machines).

### Phase 3: Aletheia Synthesis (The Sovereignty Filter)
Filter the findings through the Sovereign lens:
- **Innovation Vector**: What specific technical "trick" or architecture does this repo use to solve a problem?
- **Sovereign Compatibility**: Does this pattern rely on "Magic" (heavy frameworks/proprietary clouds) or "Logic" (deterministic, portable code)?
- **The 4-Pillar Fit**: Where would this logic live in the Tadpole OS? (Gateway, Runner, Registry, or Security).

### Phase 4: Knowledge Integration
Ensure the intelligence is permanently captured:
- **Dossier Generation**: Save a full technical breakdown to `.tmp/scout/[repo_name].md`.
- **Registry Update**: Call `share_finding` to alert other agents of the discovered pattern.
- **Pipeline Trigger**: Suggest a **[/brainstorm](brainstorm.md)** session to integrate the discovered pattern into the current project.

---

## 📊 Intelligence Dossier Format

```markdown
## 🕵️ Sovereign Scout Report: [Repo Name]
**Target**: [URL] | **Status**: [Active/Legacy] | **Sovereignty Score**: [1-10]

### 🏗️ Technical Anatomy
- **Core Stack**: [e.g., Rust 1.80 / Axum / SQLite]
- **Key Dependencies**: [List the 3 most impactful libraries]
- **Architecture Style**: [e.g., Layered, Hexagonal, Event-Driven]

### ⚡ Intelligence Vectors (The "Win")
**Pattern [Name]**: [Detailed description of the technical approach]
- **Why it works**: [The logic behind the efficiency]
- **Sovereign Adaptation**: [How to implement this without the external bloat]

### ⚠️ Risk Assessment
- **Complexity**: [Low | Medium | High]
- **Fragility**: [Does it rely on a specific, unstable environment?]
- **Licensing**: [MIT/Apache $\rightarrow$ Safe | GPL $\rightarrow$ Caution]

### 🎯 Strategic Recommendation
**Verdict**: [ADOPT | ADAPT | IGNORE]
**Next Step**: [e.g., "Trigger /brainstorm to implement this pattern into the Registry logic."]
```

---

## 💡 Usage Examples

- `/github-scout https://github.com/example/fast-rust-db` $\rightarrow$ (Analyzes DB patterns $\rightarrow$ Synthesis $\rightarrow$ Suggests `/brainstorm` for Registry update)
- `/github-scout "search:rust-websocket-framework"` $\rightarrow$ (Scouts multiple repos $\rightarrow$ Compares patterns $\rightarrow$ Recommends the most "Sovereign" one)

## 🚫 Guardrails
- **No README Reliance**: Never cite a README as the *sole* source of architectural truth. Always verify via the `src/` directory.
- **Anti-Bloat Bias**: Flag any pattern that requires the installation of excessive "helper" libraries as a **Sovereignty Risk**.
- **Deterministic Reporting**: Avoid words like "amazing" or "powerful." Use "efficient," "deterministic," "scalable," or "modular."

[//]: # (Metadata: [github_scout])
```
