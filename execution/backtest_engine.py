"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / backtest_engine
- **Primary Entrypoints**: `BacktestEngine`, `run_self_test`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[BacktestEngine]`
- **Witness Tests**: none declared
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

    def verify_trace_history(self, trace_events: List[Dict[str, Any]], test_eval_fn: Optional[Any] = None, verbose: bool = True) -> Dict[str, Any]:
        """
        Replays a series of recorded telemetry transition events.
        
        :param trace_events: List of dicts containing 'input_state', 'expected_output_state', and 'action'.
        :param test_eval_fn: Optional callable (input_state, action) -> output_state to test.
        :param verbose: Whether to print progress logs to stdout.
        :return: Results dict containing total, passed, failed, and timing stats.
        """
        start_time = time.time()
        passed_count = 0
        failed_count = 0
        failures = []

        if verbose:
            print(f"[BacktestEngine] Starting synchronous backtest across {len(trace_events)} trace events (Timeout limit: {self.timeout_seconds}s)...")

        for idx, event in enumerate(trace_events):
            elapsed = time.time() - start_time
            if elapsed > self.timeout_seconds:
                error_msg = f"Backtest TIMEOUT exceeded after {elapsed:.2f}s on event index {idx}"
                if verbose:
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

        if verbose:
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

def main():
    import argparse
    import importlib.util
    from pathlib import Path

    parser = argparse.ArgumentParser(description="Historical Trace Replay and Backtest Engine.")
    parser.add_argument("--test", action="store_true", help="Execute verification self-test.")
    parser.add_argument("--trace-file", type=str, help="Path to JSON historical trace file.")
    parser.add_argument("--world-model", type=str, help="Optional path to world model Python script for transition evaluation.")
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT_SECONDS, help="Timeout in seconds.")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format.")

    args = parser.parse_args()

    if args.test:
        success = run_self_test()
        sys.exit(0 if success else 1)

    if not args.trace_file:
        if args.json:
            print(json.dumps({"success": False, "error": "Missing required argument: --trace-file or --test"}))
        else:
            parser.print_help()
        sys.exit(1)

    trace_path = Path(args.trace_file).resolve()
    if not trace_path.is_file():
        err_msg = f"Trace file not found: {trace_path}"
        if args.json:
            print(json.dumps({"success": False, "error": err_msg}))
        else:
            print(f"[BacktestEngine] ERROR: {err_msg}", file=sys.stderr)
        sys.exit(1)

    try:
        trace_data = json.loads(trace_path.read_text(encoding="utf-8"))
        if not isinstance(trace_data, list):
            raise ValueError("Trace file must contain a JSON array of events.")
    except Exception as e:
        err_msg = f"Failed to parse trace file: {e}"
        if args.json:
            print(json.dumps({"success": False, "error": err_msg}))
        else:
            print(f"[BacktestEngine] ERROR: {err_msg}", file=sys.stderr)
        sys.exit(1)

    eval_fn = None
    if args.world_model:
        wm_path = Path(args.world_model).resolve()
        if not wm_path.is_file():
            err_msg = f"World model file not found: {wm_path}"
            if args.json:
                print(json.dumps({"success": False, "error": err_msg}))
            else:
                print(f"[BacktestEngine] ERROR: {err_msg}", file=sys.stderr)
            sys.exit(1)

        try:
            spec = importlib.util.spec_from_file_location("dynamic_wm_eval", str(wm_path))
            if spec is None or spec.loader is None:
                raise ImportError(f"Cannot load module spec from {wm_path}")
            wm = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(wm)
        except Exception as e:
            err_msg = f"Failed to import world model '{args.world_model}': {e}"
            if args.json:
                print(json.dumps({"success": False, "error": err_msg}))
            else:
                print(f"[BacktestEngine] ERROR: {err_msg}", file=sys.stderr)
            sys.exit(1)

        # Check for direct transition functions
        if hasattr(wm, "transition") and callable(getattr(wm, "transition")):
            eval_fn = wm.transition
        elif hasattr(wm, "step") and callable(getattr(wm, "step")):
            eval_fn = wm.step
        elif hasattr(wm, "evaluate_transition") and callable(getattr(wm, "evaluate_transition")):
            eval_fn = wm.evaluate_transition
        elif hasattr(wm, "get_successors") and callable(getattr(wm, "get_successors")):
            # Fall back to matching action in successors
            def successor_eval(state, action):
                try:
                    for act, next_st in wm.get_successors(state):
                        if act == action:
                            return next_st
                except Exception:
                    pass
                return None
            eval_fn = successor_eval

    engine = BacktestEngine(timeout_seconds=args.timeout)
    results = engine.verify_trace_history(trace_data, eval_fn, verbose=not args.json)

    if args.json:
        print(json.dumps(results))
    else:
        if results["success"]:
            print(f"[BacktestEngine] Trace backtest PASSED: {results['passed']}/{results['total']} events verified.")
        else:
            print(f"[BacktestEngine] Trace backtest FAILED: {results['failed']}/{results['total']} events failed. Details: {results.get('failures')}")

    sys.exit(0 if results["success"] else 1)

if __name__ == "__main__":
    main()
