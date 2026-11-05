# World Model Synthesis & Graph Search Templates (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / world-model-synthesis / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Model divergence, non-deterministic state transitions, infinite search expansion.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document provides production-grade templates and schemas for synthesizing programmatic world models and executing deterministic graph search.

---

## 1. Specification Contract for `.tmp/world_model.py`

Every synthesized world model must implement the core functional interface:

| Function | Signature | Mandatory? | Description |
|---|---|---|---|
| `get_initial_state()` | `() -> dict \| list \| primitive` | **Yes** | Returns the current state of the environment. Must be JSON-serializable. |
| `is_target_state(state)` | `(state) -> bool` | **Yes** | Returns `True` if the candidate state satisfies the mission goal. |
| `get_successors(state)` | `(state) -> list[tuple[str, Any]]` or `list[tuple[str, Any, float]]` | **Yes** | Returns valid transitions `(action_name, next_state)` or `(action_name, next_state, step_cost)`. |
| `get_heuristic(state)` | `(state) -> float` | Optional | Admissible heuristic $h(s) \le h^*(s)$ for A* search. Lower is better. |
| `transition(state, action)` | `(state, action) -> Any` | Optional | Direct deterministic state transition evaluator used by `backtest_engine.py`. |

### Invariants:
1. **Purity**: State transition functions must be pure and free of live network, database, or filesystem side-effects.
2. **State Serialization**: States are serialized using `json.dumps(state, sort_keys=True)`. Do not store unhashable or non-JSON objects (such as open file handles, sockets, or functions) in state dictionaries.

---

## 2. Template 1: Safe Database Schema Migration

```python
"""
World Model: Zero-Downtime Database Column Migration
State Keys:
- step: int (0..4)
- column_added: bool
- backfilled: bool
- dual_writing: bool
- read_swapped: bool
- old_column_dropped: bool
- rollback_ready: bool
"""

def get_initial_state():
    return {
        "step": 0,
        "column_added": False,
        "backfilled": False,
        "dual_writing": False,
        "read_swapped": False,
        "old_column_dropped": False,
        "rollback_ready": True
    }

def is_target_state(state):
    return (
        state["column_added"] and
        state["backfilled"] and
        state["dual_writing"] and
        state["read_swapped"] and
        state["old_column_dropped"]
    )

def get_successors(state):
    actions = []
    
    # Phase 1: Add nullable column
    if not state["column_added"]:
        next_s = dict(state)
        next_s["column_added"] = True
        next_s["step"] = 1
        actions.append(("ADD_NULLABLE_COLUMN", next_s, 1.0))

    # Phase 2: Start dual writing
    elif state["column_added"] and not state["dual_writing"]:
        next_s = dict(state)
        next_s["dual_writing"] = True
        next_s["step"] = 2
        actions.append(("ENABLE_DUAL_WRITING", next_s, 1.0))

    # Phase 3: Backfill historical rows
    elif state["dual_writing"] and not state["backfilled"]:
        next_s = dict(state)
        next_s["backfilled"] = True
        next_s["step"] = 3
        actions.append(("BACKFILL_HISTORICAL_DATA", next_s, 2.0))

    # Phase 4: Swap reads to new column
    elif state["backfilled"] and not state["read_swapped"]:
        next_s = dict(state)
        next_s["read_swapped"] = True
        next_s["step"] = 4
        actions.append(("SWAP_READ_TRAFFIC", next_s, 1.0))

    # Phase 5: Drop legacy column
    elif state["read_swapped"] and not state["old_column_dropped"]:
        next_s = dict(state)
        next_s["old_column_dropped"] = True
        next_s["rollback_ready"] = False
        next_s["step"] = 5
        actions.append(("DROP_OLD_COLUMN", next_s, 1.0))

    return actions

def get_heuristic(state):
    # Goal distance estimate (remaining false flags)
    flags = [
        state["column_added"],
        state["dual_writing"],
        state["backfilled"],
        state["read_swapped"],
        state["old_column_dropped"]
    ]
    return float(flags.count(False))
```

---

## 3. Template 2: Deployment Rollout with Health Gates

```python
"""
World Model: Canary Deployment Rollout
State Keys:
- canary_traffic_pct: int (0, 10, 50, 100)
- health_check_passed: bool
- p99_latency_ms: int
- error_rate_pct: float
"""

def get_initial_state():
    return {
        "canary_traffic_pct": 0,
        "health_check_passed": False,
        "p99_latency_ms": 120,
        "error_rate_pct": 0.01
    }

def is_target_state(state):
    return state["canary_traffic_pct"] == 100 and state["health_check_passed"]

def get_successors(state):
    actions = []
    pct = state["canary_traffic_pct"]

    if pct == 0:
        next_s = dict(state)
        next_s["canary_traffic_pct"] = 10
        actions.append(("DEPLOY_CANARY_10", next_s, 1.0))

    elif pct == 10:
        # Action 1: Verify health gate
        if not state["health_check_passed"]:
            next_s = dict(state)
            next_s["health_check_passed"] = True
            actions.append(("VERIFY_CANARY_HEALTH", next_s, 1.0))
        else:
            # Action 2: Advance to 50%
            next_s = dict(state)
            next_s["canary_traffic_pct"] = 50
            next_s["health_check_passed"] = False
            actions.append(("PROMOTE_CANARY_50", next_s, 1.0))

    elif pct == 50:
        if not state["health_check_passed"]:
            next_s = dict(state)
            next_s["health_check_passed"] = True
            actions.append(("VERIFY_MIDSTAGE_HEALTH", next_s, 1.0))
        else:
            next_s = dict(state)
            next_s["canary_traffic_pct"] = 100
            actions.append(("PROMOTE_FULL_TRAFFIC_100", next_s, 1.0))

    return actions
```

---

## 4. Historical Trace Format (`.tmp/history.json`)

When backtesting a world model against observed transitions, the trace file must contain a JSON array of event objects:

```json
[
  {
    "input_state": {
      "step": 0,
      "column_added": false
    },
    "action": "ADD_NULLABLE_COLUMN",
    "expected_output_state": {
      "step": 1,
      "column_added": true
    }
  },
  {
    "input_state": {
      "step": 1,
      "column_added": true
    },
    "action": "ENABLE_DUAL_WRITING",
    "expected_output_state": {
      "step": 2,
      "column_added": true,
      "dual_writing": true
    }
  }
]
```

---

## 5. Discriminative Probing Protocol (Hypothesis Falsification)

When real execution deviates from the world model:
1. **Halt Execution Immediately**: Do not continue the planned sequence.
2. **Formulate Competing Hypotheses**:
   - $H_1$: The environment has unexpected locks or transient timeouts.
   - $H_2$: A prerequisite step failed silently or produced a modified state.
3. **Execute Discriminator Probe**: Run a minimal, non-destructive read query or dry-run verification command that produces output distinguishing $H_1$ from $H_2$.
4. **Update & Anneal World Model**: Incorporate the discovered branching precondition into `get_successors(state)` and rerun the planner.
