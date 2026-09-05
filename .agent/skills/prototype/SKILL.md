---
name: prototype
description: Builds minimal, low-fidelity executable spikes or stub components to resolve "how should it look/behave" questions before production coding.
when_to_use: "Use when evaluating UI layout ideas, testing interaction behavior, or proving a concept before full implementation."
allowed-tools: Read, Glob, Grep, Write, Edit, Bash
disable-model-invocation: true
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / prototype
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Executable Spiking & UX Prototyping

Raise discussion fidelity by constructing cheap, minimal, concrete prototypes to react to—UI stubs, mock data flows, or standalone logic spikes.

---

## Operating Protocol

1. **Identify the Core Uncertainty**:
   Is the question "how should it look" (UI layout) or "how should it behave" (state transition)?
2. **Build Minimal Executable Spike**:
   - Save UI stubs or mock spikes under `.tmp/prototypes/<name>/`.
   - Avoid production dependencies, complex database persistence, or heavy styling.
3. **Verify with User**:
   Demonstrate the prototype (via preview command or screenshot/recording) to lock in design choices.
4. **Transition to Production**:
   Once approved, feed lessons learned into `/plan` and `/to-tickets` for production implementation.