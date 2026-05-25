> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[anneal]` in audit logs.
>
> ### AI Assist Note
> 🧬 Anneal Workflow: Sovereign Hardening
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🧬 Anneal Workflow: Sovereign Hardening

This workflow evaluates captured faults (Input -> Bad Output -> Desired Output) and proposes directive updates to harden the OS instruction-set.

## Steps

### 1. Evaluate Faults
Run the evaluation script to transform unresolved faults into directive proposals.

// turbo
```powershell
python execution/evaluate_annealing.py
```

### 2. Review Proposals
List the pending annealing proposals for review.

// turbo
```powershell
sqlite3 data/tadpole.db "SELECT id, name, description, status FROM capability_proposals WHERE cap_type = 'anneal' AND status = 'pending';"
```

### 3. Application
To apply a proposal, use the internal approval mechanism (or manual edit).
> [!TIP]
> Each proposal contains a `proposed_change` block. Review it carefully before merging into the core directives.

[//]: # (Metadata: [anneal])
