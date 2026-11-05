#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / world-model-synthesis / world_model_search
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import json
import argparse
import importlib.util
from pathlib import Path
from collections import deque
import heapq

# Reconfigure stdout for UTF-8 on Windows environments
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')


def load_module_from_path(file_path: Path):
    """Dynamically load a python module from file path."""
    module_name = file_path.stem
    spec = importlib.util.spec_from_file_location(module_name, str(file_path))
    if spec is None or spec.loader is None:
        raise ImportError(f"Could not load module from {file_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_world_model_interface(module) -> tuple[bool, list[str]]:
    """Validate that the world model implements required interface contracts."""
    errors = []
    required_funcs = ["get_initial_state", "is_target_state", "get_successors"]
    for func_name in required_funcs:
        if not hasattr(module, func_name) or not callable(getattr(module, func_name)):
            errors.append(f"Missing required function: {func_name}()")

    if errors:
        return False, errors

    # Check initial state serializability
    try:
        init_state = module.get_initial_state()
        json.dumps(init_state, sort_keys=True)
    except Exception as e:
        errors.append(f"get_initial_state() output is not JSON serializable: {e}")

    # Check is_target_state signature
    try:
        is_target = module.is_target_state(init_state)
        if not isinstance(is_target, bool):
            errors.append("is_target_state() must return a boolean")
    except Exception as e:
        errors.append(f"is_target_state() raised exception with initial state: {e}")

    # Check get_successors output contract
    try:
        successors = module.get_successors(init_state)
        if not isinstance(successors, list):
            errors.append("get_successors() must return a list of transitions")
        else:
            for item in successors:
                if not (isinstance(item, (tuple, list)) and len(item) in (2, 3)):
                    errors.append(f"Invalid transition tuple structure: {item}")
                    break
    except Exception as e:
        errors.append(f"get_successors() raised exception with initial state: {e}")

    return len(errors) == 0, errors


def bfs_search(module, max_depth: int = 1000, max_visited: int = 50000):
    """Perform deterministic Breadth-First Search on the world model."""
    init_state = module.get_initial_state()
    init_key = json.dumps(init_state, sort_keys=True)

    if module.is_target_state(init_state):
        return {"success": True, "path": [], "depth": 0, "visited": 1}

    queue = deque([(init_state, [])])
    visited = {init_key}

    while queue:
        state, path = queue.popleft()
        if len(path) >= max_depth:
            continue

        for transition in module.get_successors(state):
            action = transition[0]
            next_state = transition[1]
            next_key = json.dumps(next_state, sort_keys=True)

            new_path = path + [action]
            if module.is_target_state(next_state):
                return {"success": True, "path": new_path, "depth": len(new_path), "visited": len(visited)}

            if next_key not in visited:
                visited.add(next_key)
                if len(visited) > max_visited:
                    return {"success": False, "error": "max_visited_exceeded", "visited": len(visited)}
                queue.append((next_state, new_path))

    return {"success": False, "error": "no_path_found", "visited": len(visited)}


def astar_search(module, max_visited: int = 50000):
    """Perform deterministic A* Search using get_heuristic."""
    init_state = module.get_initial_state()
    init_key = json.dumps(init_state, sort_keys=True)

    if module.is_target_state(init_state):
        return {"success": True, "path": [], "cost": 0.0, "visited": 1}

    heuristic = getattr(module, "get_heuristic", lambda s: 0.0)

    # Priority queue item: (priority, counter, state, path, cost)
    counter = 0
    pq = [(heuristic(init_state), counter, init_state, [], 0.0)]
    best_costs = {init_key: 0.0}

    while pq:
        _, _, state, path, g_cost = heapq.heappop(pq)
        state_key = json.dumps(state, sort_keys=True)

        if g_cost > best_costs.get(state_key, float('inf')):
            continue

        if module.is_target_state(state):
            return {"success": True, "path": path, "cost": g_cost, "visited": len(best_costs)}

        for transition in module.get_successors(state):
            action = transition[0]
            next_state = transition[1]
            step_cost = transition[2] if len(transition) >= 3 else 1.0

            next_key = json.dumps(next_state, sort_keys=True)
            tentative_g = g_cost + step_cost

            if tentative_g < best_costs.get(next_key, float('inf')):
                best_costs[next_key] = tentative_g
                if len(best_costs) > max_visited:
                    return {"success": False, "error": "max_visited_exceeded", "visited": len(best_costs)}
                h = heuristic(next_state)
                counter += 1
                heapq.heappush(pq, (tentative_g + h, counter, next_state, path + [action], tentative_g))

    return {"success": False, "error": "no_path_found", "visited": len(best_costs)}


def run_self_tests():
    """Verify world model validation and search solver."""
    # Test 1: Mock Valid World Model
    class MockModel:
        @staticmethod
        def get_initial_state():
            return {"step": 0}

        @staticmethod
        def is_target_state(state):
            return state.get("step") == 3

        @staticmethod
        def get_successors(state):
            s = state["step"]
            if s < 3:
                return [(f"STEP_{s+1}", {"step": s + 1}, 1.0)]
            return []

        @staticmethod
        def get_heuristic(state):
            return max(0.0, 3.0 - state.get("step", 0))

    valid, errors = validate_world_model_interface(MockModel)
    assert valid, f"MockModel validation failed: {errors}"

    bfs_res = bfs_search(MockModel)
    assert bfs_res["success"], f"BFS search failed: {bfs_res}"
    assert bfs_res["path"] == ["STEP_1", "STEP_2", "STEP_3"], f"Unexpected BFS path: {bfs_res['path']}"

    astar_res = astar_search(MockModel)
    assert astar_res["success"], f"A* search failed: {astar_res}"
    assert astar_res["path"] == ["STEP_1", "STEP_2", "STEP_3"], f"Unexpected A* path: {astar_res['path']}"

    print("[PASS] world_model_search self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="World Model Search & Validator")
    parser.add_argument("model_file", nargs="?", help="Path to Python world model file")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--algorithm", choices=["auto", "bfs", "astar"], default="auto")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")

    args = parser.parse_args()

    if args.test:
        sys.exit(run_self_tests())

    if not args.model_file:
        parser.print_help()
        sys.exit(1)

    model_path = Path(args.model_file)
    if not model_path.exists():
        print(f"Error: Model file '{model_path}' not found.", file=sys.stderr)
        sys.exit(1)

    try:
        module = load_module_from_path(model_path)
    except Exception as e:
        print(f"Error importing model '{model_path}': {e}", file=sys.stderr)
        sys.exit(1)

    valid, errors = validate_world_model_interface(module)
    if not valid:
        if args.json:
            print(json.dumps({"valid": False, "errors": errors}))
        else:
            print("[FAIL] World Model interface validation errors:")
            for err in errors:
                print(f"  - {err}")
        sys.exit(1)

    has_heuristic = hasattr(module, "get_heuristic") and callable(getattr(module, "get_heuristic"))
    algo = args.algorithm
    if algo == "auto":
        algo = "astar" if has_heuristic else "bfs"

    if algo == "astar":
        result = astar_search(module)
    else:
        result = bfs_search(module)

    result["algorithm"] = algo
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        if result.get("success"):
            print(f"[SUCCESS] Search completed using {algo.upper()}.")
            print(f"Actions ({len(result['path'])} steps): {' -> '.join(result['path'])}")
            print(f"States visited: {result['visited']}")
        else:
            print(f"[FAIL] Search failed: {result.get('error')}")


if __name__ == "__main__":
    main()
