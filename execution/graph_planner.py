"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / graph_planner
- **Primary Entrypoints**: `GraphPlanner`, `run_self_test`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[GraphPlanner]`
- **Witness Tests**: none declared
"""

import sys
import os
import json
import collections
import heapq
import itertools
import importlib.util
import argparse
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple, Callable

class GraphPlanner:
    """
    Deterministic Graph Search Solver (BFS & A*) for Programmatic World Models.
    Inspired by Schema Harness ARC-AGI-3 solver methodology.
    """

    @staticmethod
    def freeze(state: Any) -> str:
        """Helper to serialize state for deterministic set/dict lookup."""
        if isinstance(state, (dict, list)):
            return json.dumps(state, sort_keys=True)
        return str(state)

    @staticmethod
    def bfs_search(
        initial_state: Any,
        is_target_fn: Callable[[Any], bool],
        get_successors_fn: Callable[[Any], List[Tuple[str, Any]]],
        max_depth: int = 1000,
        max_visited: int = 100000,
        verbose: bool = True
    ) -> Optional[List[str]]:
        """
        Executes Breadth-First Search (BFS) to find the shortest action sequence to target state.
        
        :param initial_state: Serialized or immutable representation of initial state.
        :param is_target_fn: Predicate returning True if state satisfies target goal.
        :param get_successors_fn: Function returning list of (action_name, next_state).
        :param max_depth: Depth limit for search tree.
        :param max_visited: Maximum state node threshold to prevent memory exhaustion.
        :param verbose: Whether to print exploration summary.
        :return: List of action strings if path found, else None.
        """
        queue = collections.deque([(initial_state, [])])
        visited = set()
        visited.add(GraphPlanner.freeze(initial_state))
        nodes_explored = 0

        while queue:
            current_state, path = queue.popleft()
            nodes_explored += 1

            if is_target_fn(current_state):
                if verbose:
                    print(f"[GraphPlanner] Target reached! Explored {nodes_explored} states. Path length: {len(path)}")
                return path

            if len(path) >= max_depth or nodes_explored >= max_visited:
                continue

            try:
                successors = get_successors_fn(current_state)
            except Exception as e:
                if verbose:
                    print(f"[GraphPlanner] Warning: Exception during successor expansion: {e}")
                continue

            for item in successors:
                action = item[0]
                next_state = item[1]
                frozen = GraphPlanner.freeze(next_state)
                if frozen not in visited:
                    visited.add(frozen)
                    queue.append((next_state, path + [action]))

        if verbose:
            print(f"[GraphPlanner] Search exhausted. Explored {nodes_explored} states. Target unreachable.")
        return None

    @staticmethod
    def astar_search(
        initial_state: Any,
        is_target_fn: Callable[[Any], bool],
        get_successors_fn: Callable[[Any], List[Any]],
        heuristic_fn: Optional[Callable[[Any], float]] = None,
        max_depth: int = 1000,
        max_visited: int = 100000,
        verbose: bool = True
    ) -> Optional[List[str]]:
        """
        Executes A* Search with admissible heuristic to find optimal action sequence.
        
        :param initial_state: Initial state representation.
        :param is_target_fn: Predicate returning True if target state reached.
        :param get_successors_fn: Returns list of (action, next_state) or (action, next_state, step_cost).
        :param heuristic_fn: Admissible heuristic function h(state) -> float.
        :param max_depth: Max path depth limit.
        :param max_visited: Max state exploration limit.
        :param verbose: Whether to print progress.
        :return: List of action strings if path found, else None.
        """
        counter = itertools.count()
        initial_frozen = GraphPlanner.freeze(initial_state)
        cost_so_far: Dict[str, float] = {initial_frozen: 0.0}
        
        initial_h = heuristic_fn(initial_state) if heuristic_fn else 0.0
        pq: List[Tuple[float, float, int, Any, List[str]]] = [
            (initial_h, 0.0, next(counter), initial_state, [])
        ]
        nodes_explored = 0

        while pq:
            priority, current_cost, _, current_state, path = heapq.heappop(pq)
            nodes_explored += 1

            if is_target_fn(current_state):
                if verbose:
                    print(f"[GraphPlanner:A*] Target reached! Explored {nodes_explored} states. Cost: {current_cost:.2f}. Path length: {len(path)}")
                return path

            if len(path) >= max_depth or nodes_explored >= max_visited:
                continue

            try:
                successors = get_successors_fn(current_state)
            except Exception as e:
                if verbose:
                    print(f"[GraphPlanner:A*] Warning: Exception during successor expansion: {e}")
                continue

            for item in successors:
                action = item[0]
                next_state = item[1]
                step_cost = float(item[2]) if len(item) > 2 else 1.0
                new_cost = current_cost + step_cost
                frozen = GraphPlanner.freeze(next_state)

                if frozen not in cost_so_far or new_cost < cost_so_far[frozen]:
                    cost_so_far[frozen] = new_cost
                    h = heuristic_fn(next_state) if heuristic_fn else 0.0
                    f_score = new_cost + h
                    heapq.heappush(pq, (f_score, new_cost, next(counter), next_state, path + [action]))

        if verbose:
            print(f"[GraphPlanner:A*] Search exhausted. Explored {nodes_explored} states. Target unreachable.")
        return None

def load_world_model(module_path: str):
    """Dynamically imports a world model python file."""
    path = Path(module_path).resolve()
    if not path.is_file():
        raise FileNotFoundError(f"World model file not found: {path}")

    spec = importlib.util.spec_from_file_location("dynamic_world_model", str(path))
    if spec is None or spec.loader is None:
        raise ImportError(f"Cannot load module spec from {path}")

    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def run_self_test() -> bool:
    """Executes verification self-test for GraphPlanner (BFS and A*)."""
    print("[GraphPlanner] Running self-test verification...")

    # Mock environment: Simple grid navigation (0,0) -> (2,2)
    initial_state = (0, 0)
    target_state = (2, 2)

    def is_target(s: Tuple[int, int]) -> bool:
        return s == target_state

    def get_successors(s: Tuple[int, int]) -> List[Tuple[str, Tuple[int, int]]]:
        x, y = s
        moves = []
        if x < 3: moves.append(("MOVE_RIGHT", (x + 1, y)))
        if y < 3: moves.append(("MOVE_DOWN", (x, y + 1)))
        return moves

    def manhattan_heuristic(s: Tuple[int, int]) -> float:
        return float(abs(target_state[0] - s[0]) + abs(target_state[1] - s[1]))

    # Test 1: BFS
    bfs_path = GraphPlanner.bfs_search(initial_state, is_target, get_successors, verbose=False)
    expected_length = 4
    if not bfs_path or len(bfs_path) != expected_length:
        print(f"[GraphPlanner] Self-test BFS FAILED! Expected length {expected_length}, got {bfs_path}")
        return False

    # Test 2: A*
    astar_path = GraphPlanner.astar_search(initial_state, is_target, get_successors, heuristic_fn=manhattan_heuristic, verbose=False)
    if not astar_path or len(astar_path) != expected_length:
        print(f"[GraphPlanner] Self-test A* FAILED! Expected length {expected_length}, got {astar_path}")
        return False

    print(f"[GraphPlanner] Self-test PASSED! Optimal BFS path: {bfs_path}, A* path: {astar_path}")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic Graph Search Solver for Programmatic World Models.")
    parser.add_argument("--test", action="store_true", help="Execute self-test verification suite.")
    parser.add_argument("--world-model", type=str, help="Path to Python world model script.")
    parser.add_argument("--algorithm", choices=["bfs", "astar", "auto"], default="auto", help="Search algorithm.")
    parser.add_argument("--max-depth", type=int, default=1000, help="Maximum search tree depth limit.")
    parser.add_argument("--max-visited", type=int, default=100000, help="Maximum state nodes visited threshold.")
    parser.add_argument("--output-plan", type=str, help="Optional output path to write plan JSON.")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format.")

    args = parser.parse_args()

    if args.test:
        success = run_self_test()
        sys.exit(0 if success else 1)

    if not args.world_model:
        if args.json:
            print(json.dumps({"success": False, "error": "Missing required argument: --world-model or --test"}))
        else:
            parser.print_help()
        sys.exit(1)

    try:
        wm = load_world_model(args.world_model)
    except Exception as e:
        err_msg = f"Failed to load world model '{args.world_model}': {e}"
        if args.json:
            print(json.dumps({"success": False, "error": err_msg}))
        else:
            print(f"[GraphPlanner] ERROR: {err_msg}", file=sys.stderr)
        sys.exit(1)

    # Validate required world model interface
    for fn_name in ["get_initial_state", "is_target_state", "get_successors"]:
        if not hasattr(wm, fn_name) or not callable(getattr(wm, fn_name)):
            err_msg = f"World model missing required callable: {fn_name}()"
            if args.json:
                print(json.dumps({"success": False, "error": err_msg}))
            else:
                print(f"[GraphPlanner] ERROR: {err_msg}", file=sys.stderr)
            sys.exit(1)

    initial_state = wm.get_initial_state()
    is_target_fn = wm.is_target_state
    get_successors_fn = wm.get_successors
    heuristic_fn = getattr(wm, "get_heuristic", None)

    algorithm = args.algorithm
    if algorithm == "auto":
        algorithm = "astar" if callable(heuristic_fn) else "bfs"

    verbose = not args.json
    if algorithm == "astar":
        path = GraphPlanner.astar_search(
            initial_state=initial_state,
            is_target_fn=is_target_fn,
            get_successors_fn=get_successors_fn,
            heuristic_fn=heuristic_fn,
            max_depth=args.max_depth,
            max_visited=args.max_visited,
            verbose=verbose
        )
    else:
        path = GraphPlanner.bfs_search(
            initial_state=initial_state,
            is_target_fn=is_target_fn,
            get_successors_fn=get_successors_fn,
            max_depth=args.max_depth,
            max_visited=args.max_visited,
            verbose=verbose
        )

    success = (path is not None)
    result_data = {
        "success": success,
        "algorithm": algorithm,
        "path": path if path is not None else [],
        "path_length": len(path) if path is not None else 0,
        "error": None if success else "Target state unreachable within bounds"
    }

    if args.output_plan:
        out_path = Path(args.output_plan)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(result_data, indent=2), encoding="utf-8")
        if verbose:
            print(f"[GraphPlanner] Plan written to {out_path}")

    if args.json:
        print(json.dumps(result_data))
    else:
        if success:
            print(f"[GraphPlanner] Plan generated successfully: {path}")
        else:
            print("[GraphPlanner] Target state unreachable.")

    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
