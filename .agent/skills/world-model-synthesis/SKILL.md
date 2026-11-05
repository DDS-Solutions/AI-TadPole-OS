---
name: world-model-synthesis
description: Synthesizes programmatic executable world models (.tmp/world_model.py) and executes local graph search (BFS/A*) for deterministic agent task planning.
when_to_use: "When planning complex multi-step state mutations (>=3 steps), database schema migrations, deployment rollouts, infrastructure topology transitions, or any operation with rollback risks and non-linear dependencies. NOT for single-step commands or simple linear scripts."
allowed-tools: Read, Write, Bash, RunCommand
version: "2.1.0"
priority: high
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / world-model-synthesis
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Model divergence, non-deterministic state transitions, infinite search expansion.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# World Model Synthesis & Graph Search Planning Protocol

> **Knowledge Heritage**: Inspired by Schema Harness (ARC-AGI-3 ~99% Public benchmark).  
> **Source of Truth**: `execution/graph_planner.py`, `execution/backtest_engine.py`

## Overview
When agents encounter complex multi-step environment state changes (e.g. database schema migrations, deployment rollouts, infrastructure topology transitions), agents **MUST NOT** guess step-by-step actions in natural language. Instead, agents synthesize a **programmatic world model** (`.tmp/world_model.py`) and execute a local deterministic graph search (BFS or A*) to compute an optimal, zero-token action sequence.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core workflow below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/world_model_templates.md`](./references/world_model_templates.md) | Production templates for zero-downtime DB migrations, canary rollouts, and trace schemas | When synthesizing multi-branch world models or backtesting historical telemetry |
| [`scripts/world_model_search.py`](./scripts/world_model_search.py) | Deterministic world model interface validator and BFS/A* local solver with `--test` suite | Automated validation of synthesized models or running local plan computations |

---

## 🧭 Decision Matrix: When to Synthesize a World Model

| Scenario | Recommend World Model? | Justification |
|---|:---:|---|
| **Multi-step state transition ($\ge 3$ dependent steps)** | **YES** | Graph search eliminates compounding reasoning errors and verifies path safety. |
| **Database schema migration or data backfill** | **YES** | Pre-computes validation checkpoints, dual-writing gates, and rollback states. |
| **Multi-stage service deployment / canary rollout** | **YES** | Enforces health-check gates and rollback transitions deterministically. |
| **Single-shot command execution (e.g. `npm test`, `git status`)** | **NO** | Overkill; run deterministic CLI commands directly. |
| **Linear script without branching or state rollback** | **NO** | Standard procedural execution in `execution/` is preferred. |
| **Unbounded / infinite continuous state space** | **NO** | Graph search requires discrete or discretized state transitions. |

---

## 📐 Invariants & Non-Negotiables

1. **State JSON-Serializability**: Every state returned by `get_initial_state()` and `get_successors()` must be serializable via `json.dumps(state, sort_keys=True)`. Do not store custom class instances, lambdas, or unhashable sets directly in state dictionaries.
2. **Side-Effect Free Evaluation**: The world model functions (`is_target_state`, `get_successors`, `get_heuristic`) must be pure. Never invoke live external network requests, database mutations, or filesystem writes inside the graph exploration loop.
3. **Exploration Bounds**: Graph search is strictly bounded by `max_depth` (default 1,000) and `max_visited` (default 100,000 states) to guarantee termination and prevent memory exhaustion.

---

## 🛡️ Socratic Gate: Synthesis Pre-Conditions & Invariant Audits

Before synthesizing a world model, confirm:
1. **Discrete State Representation**: Are all system variables expressible as discrete flags, counters, or enums?
2. **Admissibility of Heuristic**: If providing `get_heuristic(state)` for A*, is it strictly admissible ($h(s) \le h^*(s)$)?
3. **Rollback Trajectories**: Does `get_successors(state)` include compensating / abort actions from intermediate failure states?

---

## Operating Protocol

### Step 1: Synthesize World Model Script (`.tmp/world_model.py`)
Write a lightweight Python module representing the system state and transition rules:
```python
def get_initial_state():
    """Returns the starting state (JSON-serializable)."""
    return {"step": 0, "status": "PENDING"}

def is_target_state(state):
    """Predicate returning True when goal state is achieved."""
    return state.get("status") == "COMPLETED"

def get_successors(state):
    """
    Returns list of (action_name, next_state) 
    or (action_name, next_state, step_cost).
    """
    actions = []
    if state["step"] == 0:
        actions.append(("VALIDATE_SCHEMA", {"step": 1, "status": "VALIDATED"}))
    elif state["step"] == 1:
        actions.append(("APPLY_MIGRATION", {"step": 2, "status": "COMPLETED"}))
    return actions

# Optional: Provide admissible heuristic for A* search
def get_heuristic(state):
    return 1.0 if state.get("status") != "COMPLETED" else 0.0
```

### Step 2: Validate Model Interface & Search Plan
Execute the deterministic validator and local solver:
```powershell
python .agent/skills/world-model-synthesis/scripts/world_model_search.py .tmp/world_model.py --algorithm auto --json
```

### Step 3: Backtest against Recorded Telemetry
Before running actions against production or live environments, backtest the world model against historical transition traces (`.tmp/history.json`):
```powershell
python execution/backtest_engine.py --trace-file .tmp/history.json --world-model .tmp/world_model.py
```
*Add `--json` for structured LLM parsing.*

### Step 4: Run Graph Search Planner (BFS / A*)
Execute `execution/graph_planner.py` to derive the optimal zero-token action sequence:
```powershell
# Automatically selects A* if get_heuristic exists, otherwise BFS
python execution/graph_planner.py --world-model .tmp/world_model.py --algorithm auto --output-plan .tmp/plan.json
```
*Add `--json` for structured LLM parsing (`{"success": true, "path": [...], "path_length": 2}`).*

### Step 5: Execute & Falsify
1. Execute the returned action path deterministically step-by-step.
2. If real-world telemetry diverges from the model's expected state:
   - **Halt execution**: Do not force the remaining action sequence.
   - **Formulate competing hypotheses** ($H_1, H_2$) explaining why the environment diverged.
   - **Execute Discriminator Probe**: Run a targeted non-destructive verification query to falsify incorrect hypotheses.
   - **Anneal World Model**: Update `get_successors(state)` with the newly discovered precondition and replan.
   - For complex system-wide root cause investigations, trigger the Sovereign Root Cause Analysis protocol ([`/debug`](file:///d:/TadpoleOS-Dev/.agent/workflows/debug.md)).

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Non-deterministic state evaluation (calling `datetime.now()` or random) | Cycle detection fails, graph explosions | Pass current timestamp or seed in state dict |
| Performing live network or I/O calls inside `get_successors` | Massive latency, accidental side-effects during search | World model transitions must be 100% pure functions |
| Unbounded state space without termination guard | OOM / hang in infinite state exploration | Enforce `max_depth` and `max_visited` thresholds |
| Over-optimistic heuristic $h(s) > h^*(s)$ | Inadmissible heuristic produces sub-optimal plans | Verify $h(s)$ never overestimates true remaining cost |

---

## 🛠️ Verification Gate

Verify the planner engine, local search solver, and specification compliance:
```powershell
# Verify local world model validator & search tool
python .agent/skills/world-model-synthesis/scripts/world_model_search.py --test

# Verify Graph Planner solver (BFS + A*)
python execution/graph_planner.py --test

# Verify Trace Backtest Replay engine
python execution/backtest_engine.py --test

# Verify Open Specification standards compliance
python execution/verify_skill_specs.py
```