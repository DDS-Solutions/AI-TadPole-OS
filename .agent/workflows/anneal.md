---
description: Annealing is the process of hardening the Sovereign OS instruction-set by iteratively resolving "faults." In this context, a fault is defined as a discrepancy where the system's output deviates from the intended sovereign logic.
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Sovereign Workflows / anneal
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[anneal]`)
>
> ### 🤖 AI Persona Directive
> When `/anneal` is active, operate as the **Sovereign Annealing Engine / Hardening Specialist**. Your role is to remove internal stresses (logic discrepancies) from the instruction-set. Analyze fault triplets `[Input] → [Bad Output] → [Desired Output]`, extract root-cause directive deficiencies, and formulate surgical, permanent OS updates.

# 🧬 Anneal Workflow: Sovereign Hardening

**Usage**: `/anneal [fault_id | --evaluate | --apply]`

---

## 🎯 Purpose
Annealing is the process of hardening the Sovereign OS instruction-set by iteratively resolving "faults." In this context, a fault is defined as a discrepancy where the system's output deviates from the intended sovereign logic.

> 💡 **Bound Skills & References**:
> - Use [`aletheia-reasoning`](../skills/aletheia-reasoning/SKILL.md) for iterative Generator $\rightarrow$ Verifier $\rightarrow$ Reviser fault resolution loops.
> - Use [`systematic-debugging`](../skills/systematic-debugging/SKILL.md) for 4-phase root cause analysis and divergence isolation.
> - Use [`clean-code`](../skills/clean-code/SKILL.md) to ensure proposed directive updates remain surgical without over-restriction.
> - Reference active system faults in [`directives/FAULT_REGISTRY.md`](../../directives/FAULT_REGISTRY.md).

---

## The Logic Loop
The workflow operates on a **Correction Cycle**:
1. **Capture**: A fault is logged as a triplet: `[Input] → [Bad Output] → [Desired Output]`.
2. **Evaluation**: The system analyzes the delta between the *Bad* and *Desired* output to identify the missing or conflicting directive.
3. **Proposal**: A specific update to the core directives is proposed to "bridge" this gap.
4. **Hardening**: Once applied, the directive becomes a permanent part of the OS, preventing the specific class of error from recurring.

This process mimics metallurgical annealing—removing internal stresses (logic errors) and increasing the structural integrity (reliability) of the system's reasoning.

---

### 🛠 Execution Steps

### 1. Evaluate Faults
Analyze unresolved faults and generate proposed directive updates. This script parses the fault logs and suggests a `proposed_change` block.

// turbo
```powershell
python execution/evaluate_annealing.py
```

### 2. Review Proposals
Query the `tadpole.db` to inspect pending proposals. Review the `description` and the `proposed_change` to ensure the fix is surgical and does not introduce over-restriction.

// turbo
```powershell
sqlite3 data/tadpole.db "SELECT id, name, description, status FROM capability_proposals WHERE cap_type = 'anneal' AND status = 'pending';"
```

### 3. Application & Integration
Integrate the approved proposal into the core instruction set.

- **Manual Integration**: Copy the `proposed_change` block into the relevant directive file within `@docs ARCHITECTURE:Core`.
- **State Update**: Update the proposal status in the database from `pending` to `applied`.

> [!TIP]
> **Avoid Over-fitting**: Ensure the proposed change addresses the *root cause* of the fault rather than just the specific instance, ensuring the hardening applies to the entire class of error.

### 4. Validation & Parity Check
Verify that the fault is resolved and that no "regression" has occurred in other system capabilities.

1. **Re-Test**: Run the original `Input` and verify it now produces the `Desired Output`.
2. **Parity Guard**: Run the parity check to ensure core architectural stability.

// turbo
```powershell
python execution/parity_guard.py
```

---

## 🕸️ Graph Intelligence
Before modifying core directives or evaluating systemic blast radius of an annealing proposal, inspect affected symbols and dependencies:
```powershell
npm run graph:blast:guard -- --path server-rs/src/agent/workflows.rs
npm run graph:lookup -- --name evaluate_annealing
```