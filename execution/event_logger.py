#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Append-Only Action-Observation Event Logger (event_logger.py)**: Records agent actions, 
tool calls, and observations into an append-only JSONL log (`.tmp/events.jsonl`) for 
lightweight execution trace tracking, telemetry inspection, and post-mission auditability.

### 🔍 Debugging & Observability
- **Trace Scope**: `execution::event_logger`
- **Output Target**: `.tmp/events.jsonl`
"""

import sys
import os
import json
import time
from datetime import datetime, timezone
from pathlib import Path

# Add execution dir to import path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from py_utils import init_utf8, print_ok, print_warn, print_err, print_step, print_header

init_utf8()

WORKSPACE_ROOT = Path(__file__).parent.parent
TMP_DIR = WORKSPACE_ROOT / ".tmp"
EVENTS_LOG_PATH = TMP_DIR / "events.jsonl"


def ensure_tmp_dir():
    """Ensure .tmp directory exists."""
    TMP_DIR.mkdir(parents=True, exist_ok=True)


def log_event(event_type: str, action: str, observation: dict = None, status: str = "COMPLETED") -> dict:
    """
    Append a structured Action-Observation event to .tmp/events.jsonl.
    
    :param event_type: Classification of the event (e.g. 'TOOL_CALL', 'AGENT_HANDOFF', 'SCRIPT_EXEC')
    :param action: Short description of the action being performed.
    :param observation: Dictionary containing execution result telemetry or metadata.
    :param status: 'PENDING', 'COMPLETED', 'FAILED', or 'REVERTED'
    """
    ensure_tmp_dir()
    
    timestamp = datetime.now(timezone.utc).isoformat()
    record = {
        "timestamp": timestamp,
        "epoch_ms": int(time.time() * 1000),
        "event_type": event_type,
        "action": action,
        "status": status,
        "observation": observation or {}
    }
    
    try:
        with open(EVENTS_LOG_PATH, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, ensure_ascii=False) + "\n")
        return record
    except Exception as e:
        print_warn(f"Failed to write to event log: {e}")
        return record


def run_self_test() -> bool:
    """Self-test routine for event_logger.py."""
    print_header("Tadpole OS Event Logger Self-Test")
    print_step("Writing test event to .tmp/events.jsonl...")
    
    test_record = log_event(
        event_type="SELF_TEST",
        action="Executing event logger integrity check",
        observation={"test_key": "test_value", "result": "PASS"},
        status="COMPLETED"
    )
    
    if EVENTS_LOG_PATH.exists():
        print_ok(f"Event logged successfully to: {EVENTS_LOG_PATH}")
        print_ok(f"Recorded payload: {json.dumps(test_record)}")
        return True
    else:
        print_err("Event log file was not created!")
        return False


if __name__ == "__main__":
    success = run_self_test()
    sys.exit(0 if success else 1)
