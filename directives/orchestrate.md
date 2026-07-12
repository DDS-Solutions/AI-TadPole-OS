> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[orchestrate]` in audit logs.
>
> ### AI Assist Note
> Orchestration Protocol
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

# Orchestration Protocol

1. **Gap-Finding Phase (MANDATORY START):** 
   - At the beginning of the orchestration loop, explicitly state: "What is the requirement? What is the current state? What is the gap?"
2. Initialize mission parameters based on the gap.
3. Delegate 'alpha' as lead node.
4. Assign worker nodes to specialists.
5. **Linear Task List with Validation Gates:**
   - Organize tasks in `task.md` linearly.
   - Insert explicit `[ ] **GATE:** Run validation_gates.py --target <target>` items at critical milestones.
   - **CRITICAL:** Do not proceed to subsequent tasks until the current GATE is independently verified and passes.
6. Establish neural handoff channels.
7. Monitor for worker node disconnection and failed gates (Attention State).

[//]: # (Metadata: [orchestrate])
