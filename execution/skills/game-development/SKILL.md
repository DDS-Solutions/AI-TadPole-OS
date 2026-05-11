> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: game-development
description: Game development orchestrator. Routes to platform-specific skills based on project needs.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Game Development Orchestrator

**Principles + Routing.**

## Routing
- **Platform**: Web, Mobile, PC, VR/AR.
- **Dimension**: 2D, 3D.
- **Specialty**: Design, Multiplayer, Art, Audio.

## Core Principles
1.  **Game Loop**: Input -> Update (Fixed) -> Render (Fast).
2.  **Pattern**: State Machine -> Object Pooling -> ECS (if needed).
3.  **Input**: Abstract actions ("Jump", not "Space").
4.  **Perf Budget**: 16ms/frame. Profile first.

---

## 🧠 Aletheia Reasoning Protocol (Game Logic)

**Fun is subjective. Performance is objective.**

### 1. Generator (Loop Design)
*   **Core Loop**: "Kill -> Loot -> Upgrade".
*   **Juiciness**: "Where can we add screenshake? Particles? Pause frames?".
*   **Hook**: "What happens in the first 10 seconds?".

### 2. Verifier (Frame Budget)
*   **16ms Rule**: "Does this system fit in 16ms?".
*   **Memory**: "Are we allocating garbage every frame?".
*   **Fun Test**: "Is the movement satisfying without graphics?".

### 3. Reviser (Polish)
*   **Feedback**: "Add audio cue for jump".
*   **Ease**: "Add coyote time (jump after falling)".

---

## 🛡️ Security & Safety Protocol (Game)

1.  **Trust No One**: Server validates ALL movement/damage.
2.  **Econ Safety**: Virtual currency transactions = server-side atomic.
3.  **Chat**: Filter profanity/injection.
4.  **Assets**: Don't ship source files.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
