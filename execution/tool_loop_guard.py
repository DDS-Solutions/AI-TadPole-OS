#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**🛡️ Tadpole Engine: Tool Loop Guard**
Deterministic tool call hashing and circuit-breaker verification module to prevent
infinite agent execution loops and repetitive error cycles.

### 🔍 Debugging & Observability
- **Failure Path**: Duplicate tool payload threshold breached (>= 3 identical calls).
- **Telemetry Link**: Search `[tool_loop_guard]` in system telemetry.
"""

import sys
import os
import hashlib
import json
from typing import List, Dict, Any, Tuple
from pathlib import Path

# Add project root and execution directory to sys.path
script_dir = Path(__file__).parent
project_root = script_dir.parent
if str(script_dir) not in sys.path:
    sys.path.insert(0, str(script_dir))
if str(project_root) not in sys.path:
    sys.path.insert(0, str(project_root))

try:
    from py_utils import init_utf8, print_ok, print_warn, print_err
except ImportError:
    from execution.py_utils import init_utf8, print_ok, print_warn, print_err

init_utf8()

DEFAULT_REPETITION_THRESHOLD = 3

class ToolLoopGuard:
    """Tracks tool invocation hashes in a sliding window to detect infinite cycles."""
    
    def __init__(self, threshold: int = DEFAULT_REPETITION_THRESHOLD):
        self.threshold = threshold
        self.history: List[str] = []

    def compute_hash(self, tool_name: str, args: Dict[str, Any]) -> str:
        """Generate a deterministic SHA-256 hash from tool name and canonicalized JSON args."""
        canonical_args = json.dumps(args, sort_keys=True, default=str)
        payload = f"{tool_name}:{canonical_args}"
        return hashlib.sha256(payload.encode('utf-8')).hexdigest()

    def record_and_evaluate(self, tool_name: str, args: Dict[str, Any]) -> Tuple[bool, int, str]:
        """
        Record a tool call. Returns (is_tripped, repeat_count, payload_hash).
        is_tripped is True if identical call count >= threshold.
        """
        payload_hash = self.compute_hash(tool_name, args)
        self.history.append(payload_hash)
        
        repeat_count = self.history.count(payload_hash)
        is_tripped = repeat_count >= self.threshold
        
        return is_tripped, repeat_count, payload_hash

def evaluate_tool_sequence(calls: List[Dict[str, Any]], threshold: int = DEFAULT_REPETITION_THRESHOLD) -> bool:
    """
    Evaluates a sequence of tool calls.
    Returns True if sequence is healthy, False if loop circuit-breaker was tripped.
    """
    guard = ToolLoopGuard(threshold=threshold)
    for index, call in enumerate(calls):
        name = call.get("tool_name", "")
        args = call.get("args", {})
        tripped, count, payload_hash = guard.record_and_evaluate(name, args)
        if tripped:
            print_err(f"Circuit Breaker TRIPPED at index {index}! Tool '{name}' repeated {count} times (Hash: {payload_hash[:8]})")
            return False
    print_ok(f"Tool sequence clean ({len(calls)} calls evaluated, 0 loop traps).")
    return True

if __name__ == "__main__":
    # Smoke test sequence
    sample_healthy = [
        {"tool_name": "view_file", "args": {"AbsolutePath": str(project_root / ".env")}},
        {"tool_name": "view_file", "args": {"AbsolutePath": str(project_root / "src" / "App.tsx")}},
        {"tool_name": "view_file", "args": {"AbsolutePath": str(project_root / ".env")}}
    ]
    sample_loop = [
        {"tool_name": "update_mission_task", "args": {"task_id": "1", "status": "retry"}},
        {"tool_name": "update_mission_task", "args": {"task_id": "1", "status": "retry"}},
        {"tool_name": "update_mission_task", "args": {"task_id": "1", "status": "retry"}}
    ]
    
    print("Testing Healthy Sequence:")
    healthy_res = evaluate_tool_sequence(sample_healthy)
    print("\nTesting Loop Trapped Sequence:")
    loop_res = evaluate_tool_sequence(sample_loop)
    
    if healthy_res and not loop_res:
        print_ok("ToolLoopGuard self-test PASSED!")
        sys.exit(0)
    else:
        print_err("ToolLoopGuard self-test FAILED!")
        sys.exit(1)

# Metadata: [tool_loop_guard]
