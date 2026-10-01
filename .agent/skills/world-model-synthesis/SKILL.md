---
name: world-model-synthesis
description: Synthesizes programmatic executable world models (.tmp/world_model.py) and executes local graph search (BFS/A*) for deterministic agent task planning.
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / world-model-synthesis
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# World Model Synthesis & Graph Search Planning Skill

> **Knowledge Heritage**: Inspired by Schema Harness (ARC-AGI-3 ~99% Public benchmark).  
> **Source of Truth**: `execution/graph_planner.py`, `execution/backtest_engine.py`

## Overview
When agents encounter complex multi-step environment state changes (e.g. database schema migrations, deployment rollouts, infrastructure topology transitions), agents MUST synthesize a **programmatic world model** rather than guessing step-by-step actions in natural language.

---

## Operating Protocol

### Step 1: Synthesize World Model Script (`.tmp/world_model.py`)
Write a lightweight Python module representing the system state and transition rules:
```python
def get_initial_state():
    return {"step": 0, "status": "PENDING"}

def is_target_state(state):
    return state.get("status") == "COMPLETED"

def get_successors(state):
    # Returns list of (action_name, next_state)
    actions = []
    if state["step"] == 0:
        actions.append(("VALIDATE_SCHEMA", {"step": 1, "status": "VALIDATED"}))
    elif state["step"] == 1:
        actions.append(("APPLY_MIGRATION", {"step": 2, "status": "COMPLETED"}))
    return actions
```

### Step 2: Backtest against Recorded Telemetry
Run `python execution/backtest_engine.py` to backtest the world model against historical transition logs:
```powershell
python execution/backtest_engine.py --trace-file .tmp/history.json
```

### Step 3: Run Graph Search Planner
Execute `execution/graph_planner.py` to derive the optimal zero-token action sequence:
```powershell
python execution/graph_planner.py --world-model .tmp/world_model.py
```

### Step 4: Execute & Monitor
Execute the returned action path deterministically. If an unexpected state occurs, trigger the **Discriminative Probing** workflow to falsify competing hypotheses before modifying the world model.