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
import json
import collections
from typing import Dict, List, Any, Optional, Tuple, Callable

class GraphPlanner:
    """
    Deterministic Graph Search Solver (BFS & A*) for Programmatic World Models.
    Inspired by Schema Harness ARC-AGI-3 solver methodology.
    """
    
    @staticmethod
    def bfs_search(
        initial_state: Any,
        is_target_fn: Callable[[Any], bool],
        get_successors_fn: Callable[[Any], List[Tuple[str, Any]]],
        max_depth: int = 1000,
        max_visited: int = 100000
    ) -> Optional[List[str]]:
        """
        Executes Breadth-First Search (BFS) to find the shortest action sequence to target state.
        
        :param initial_state: Serialized or immutable representation of initial state.
        :param is_target_fn: Predicate returning True if state satisfies target goal.
        :param get_successors_fn: Function returning list of (action_name, next_state).
        :param max_depth: Depth limit for search tree.
        :param max_visited: Maximum state node threshold to prevent memory exhaustion.
        :return: List of action strings if path found, else None.
        """
        queue = collections.deque([(initial_state, [])])
        visited = set()
        
        # Helper to serialize state for set lookup if dict/list
        def freeze(state: Any) -> str:
            if isinstance(state, (dict, list)):
                return json.dumps(state, sort_keys=True)
            return str(state)

        visited.add(freeze(initial_state))
        nodes_explored = 0

        while queue:
            current_state, path = queue.popleft()
            nodes_explored += 1

            if is_target_fn(current_state):
                print(f"[GraphPlanner] Target reached! Explored {nodes_explored} states. Path length: {len(path)}")
                return path

            if len(path) >= max_depth or nodes_explored >= max_visited:
                continue

            try:
                successors = get_successors_fn(current_state)
            except Exception as e:
                print(f"[GraphPlanner] Warning: Exception during successor expansion: {e}")
                continue

            for action, next_state in successors:
                frozen = freeze(next_state)
                if frozen not in visited:
                    visited.add(frozen)
                    queue.append((next_state, path + [action]))

        print(f"[GraphPlanner] Search exhausted. Explored {nodes_explored} states. Target unreachable.")
        return None

def run_self_test() -> bool:
    """Executes verification self-test for GraphPlanner."""
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

    path = GraphPlanner.bfs_search(initial_state, is_target, get_successors)
    expected_length = 4
    if path and len(path) == expected_length:
        print(f"[GraphPlanner] Self-test PASSED! Optimal path: {path}")
        return True
    else:
        print(f"[GraphPlanner] Self-test FAILED! Expected length {expected_length}, got {path}")
        return False

if __name__ == "__main__":
    if "--test" in sys.argv:
        success = run_self_test()
        sys.exit(0 if success else 1)
    else:
        print("Graph Planner Utility. Use --test to execute verification suite.")
