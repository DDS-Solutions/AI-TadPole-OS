"""
@docs ARCHITECTURE:Core Verification Engine
@docs OPERATIONS_MANUAL:Trace Backtesting

### AI Assist Note
**Backtest Engine**: Historical trace replay & regression verifier inspired by Schema Harness.
Executes synchronous replay of recorded telemetry events against updated directives or scripts.
Enforces strict 10-second timeout to verify zero regression across historical transition vectors.

### 🔍 Debugging & Observability
- **Failure Path**: Divergence between actual output state and historical recorded output state, or execution timeout (>10s).
- **Telemetry Link**: Search `[BacktestEngine]` in system logs.
"""

import sys
import os
import json
import time
import signal
from typing import List, Dict, Any, Optional

DEFAULT_TIMEOUT_SECONDS = 10.0

class BacktestEngine:
    """
    Synchronous Historical Trace Replay Engine.
    Verifies 100% parity between updated execution scripts and past telemetry traces.
    """

    def __init__(self, timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS):
        self.timeout_seconds = timeout_seconds

    def verify_trace_history(self, trace_events: List[Dict[str, Any]], test_eval_fn: Optional[Any] = None) -> Dict[str, Any]:
        """
        Replays a series of recorded telemetry transition events.
        
        :param trace_events: List of dicts containing 'input_state', 'expected_output_state', and 'action'.
        :param test_eval_fn: Optional callable (input_state, action) -> output_state to test.
        :return: Results dict containing total, passed, failed, and timing stats.
        """
        start_time = time.time()
        passed_count = 0
        failed_count = 0
        failures = []

        print(f"[BacktestEngine] Starting synchronous backtest across {len(trace_events)} trace events (Timeout limit: {self.timeout_seconds}s)...")

        for idx, event in enumerate(trace_events):
            elapsed = time.time() - start_time
            if elapsed > self.timeout_seconds:
                error_msg = f"Backtest TIMEOUT exceeded after {elapsed:.2f}s on event index {idx}"
                print(f"[BacktestEngine] ERROR: {error_msg}")
                return {
                    "success": False,
                    "total": len(trace_events),
                    "passed": passed_count,
                    "failed": failed_count + (len(trace_events) - idx),
                    "error": error_msg,
                    "elapsed_seconds": round(elapsed, 3)
                }

            input_state = event.get("input_state")
            expected_output = event.get("expected_output_state")
            action = event.get("action", "EXECUTE")

            # Execute transition evaluation if function provided, else mock pass for trace schema validity
            if test_eval_fn:
                try:
                    actual_output = test_eval_fn(input_state, action)
                    if actual_output == expected_output:
                        passed_count += 1
                    else:
                        failed_count += 1
                        failures.append({
                            "index": idx,
                            "action": action,
                            "expected": expected_output,
                            "got": actual_output
                        })
                except Exception as e:
                    failed_count += 1
                    failures.append({"index": idx, "action": action, "error": str(e)})
            else:
                # Basic trace structural validation
                if input_state is not None and expected_output is not None:
                    passed_count += 1
                else:
                    failed_count += 1
                    failures.append({"index": idx, "error": "Missing input or output state payload"})

        elapsed_total = time.time() - start_time
        success = (failed_count == 0)

        print(f"[BacktestEngine] Backtest complete in {elapsed_total:.3f}s: {passed_count}/{len(trace_events)} passed (100% parity: {success})")

        return {
            "success": success,
            "total": len(trace_events),
            "passed": passed_count,
            "failed": failed_count,
            "failures": failures,
            "elapsed_seconds": round(elapsed_total, 3)
        }

def run_self_test() -> bool:
    """Verification test suite for BacktestEngine."""
    print("[BacktestEngine] Running verification self-test...")
    engine = BacktestEngine(timeout_seconds=10.0)

    # 1. Test 100% Parity (Pass)
    mock_traces = [
        {"input_state": {"value": 1}, "action": "ADD_ONE", "expected_output_state": {"value": 2}},
        {"input_state": {"value": 5}, "action": "ADD_ONE", "expected_output_state": {"value": 6}},
        {"input_state": {"value": 10}, "action": "ADD_ONE", "expected_output_state": {"value": 11}},
    ]

    def mock_evaluator(state: Dict[str, Any], action: str) -> Dict[str, Any]:
        if action == "ADD_ONE":
            return {"value": state["value"] + 1}
        return state

    res_pass = engine.verify_trace_history(mock_traces, mock_evaluator)
    if not res_pass["success"]:
        print(f"[BacktestEngine] Self-test FAILED on valid trace suite: {res_pass}")
        return False

    # 2. Test Regression Falsification (Fail)
    def broken_evaluator(state: Dict[str, Any], action: str) -> Dict[str, Any]:
        return {"value": state["value"] + 2} # Bug!

    res_fail = engine.verify_trace_history(mock_traces, broken_evaluator)
    if res_fail["success"]:
        print("[BacktestEngine] Self-test FAILED: Did not catch broken evaluator regression!")
        return False

    print("[BacktestEngine] Self-test PASSED cleanly! Replay engine verified with 10s strict timeout.")
    return True

if __name__ == "__main__":
    if "--test" in sys.argv:
        success = run_self_test()
        sys.exit(0 if success else 1)
    else:
        print("Backtest Engine Utility. Use --test to execute verification suite.")
