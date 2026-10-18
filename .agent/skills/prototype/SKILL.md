---
name: prototype
description: "Builds minimal, low-fidelity executable spikes or stub components to resolve 'how should it look/behave' questions before production coding."
when_to_use: "Use when evaluating UI layout ideas, testing interaction behavior, or proving a concept before full production implementation."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: MEDIUM
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / prototype
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Committing ephemeral prototype spikes to production directories or over-engineering preliminary spikes.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Executable Spiking & UX Prototyping

> **Purpose**: Raise discussion fidelity and resolve design ambiguities by constructing cheap, minimal, isolated executable prototypes in `.tmp/prototypes/` before committing production code.  
> **Workflow Bindings**: Integrated into [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md), [`/brainstorm`](file:///D:/TadpoleOS-Dev/.agent/workflows/brainstorm.md), and [`/ui-ux-pro-max`](file:///D:/TadpoleOS-Dev/.agent/workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/spike_scaffolds_and_sandbox.md`](file:///D:/TadpoleOS-Dev/.agent/skills/prototype/references/spike_scaffolds_and_sandbox.md) | Single-file HTML/CSS/JS templates, event listeners, and sandbox lifecycle | Scaffolding new interaction spikes |
| [`scripts/prototype_scaffolder.py`](file:///D:/TadpoleOS-Dev/.agent/skills/prototype/scripts/prototype_scaffolder.py) | Automated prototype spike scaffolder and entry point validator | Generating `.tmp/prototypes/<name>` sandboxes |

---

## 🔄 1. The 4-Phase Prototyping Protocol

```
[1. UNCERTAINTY] ───> [2. SCAFFOLD] ───> [3. DEMO] ───> [4. PRODUCTION]
 Identify look vs.    Generate minimal   Demonstrate to  Transfer lessons
 behavior question    spike in .tmp/     user & lock in  into /plan & delete
```

1. **Identify the Core Uncertainty**: Determine whether the unknown is visual layout ("how should it look") or reactive state ("how should it behave").
2. **Build Minimal Executable Spike**: Scaffolding runs exclusively under `.tmp/prototypes/<name>/`. Avoid production database dependencies or heavy stylesheets.
3. **Verify with User**: Demonstrate interaction or layout to lock in architectural and design choices.
4. **Transition to Production**: Extract proven patterns into [`/plan`](file:///D:/TadpoleOS-Dev/.agent/workflows/plan.md) and purge ephemeral prototype files.

---

## 🧠 2. Socratic Gate for Prototyping

Before building a prototype spike, answer:
- **Gate 1 (Isolation)**: Is the spike isolated inside `.tmp/prototypes/` so it never pollutes `src/` or `server-rs/`?
- **Gate 2 (Zero-Waste)**: Does the spike answer a single explicit uncertainty rather than building the whole feature prematurely?
- **Gate 3 (Throwaway Discipline)**: Is it understood that this code will be rewritten cleanly following production standards rather than copy-pasted directly?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Production Pollution** | Creating prototype `.tsx` files directly in `src/pages/`. | Confine all spikes to `.tmp/prototypes/`. |
| **Over-Engineering Spikes** | Adding full authentication, Redux/Zustand, and DB models to a button test. | Build single-file minimal HTML/JS or stub components. |
| **Shipping Prototypes as Prod** | Retaining messy spike code in production branches. | Throw away spike; reimplement cleanly via `/plan`. |

---

## 🛠️ 4. Deterministic Verification Gate

Scaffold and validate interactive prototypes:

```powershell
# 1. Run built-in self-tests
python .agent/skills/prototype/scripts/prototype_scaffolder.py --test

# 2. Scaffold a new interactive spike
python .agent/skills/prototype/scripts/prototype_scaffolder.py --scaffold "model-selector" --desc "Test dropdown reactivity"
```