#!/usr/bin/env python3
"""
### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Deterministic Python tool for cross-platform 7-day rolling telemetry log retention.

### 🔍 Debugging & Observability
Traceability via `execution/parity_guard.py`.
@docs ARCHITECTURE:Core
"""

import sys
import os
import time
import argparse
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE_ROOT / "data" / "logs"

def prune_telemetry_logs(max_days: int = 7) -> dict:
    """Delete JSONL telemetry files older than max_days."""
    if not LOG_DIR.exists():
        print(f"[LOG_CLEANUP] Log directory {LOG_DIR} does not exist.")
        return {"deleted": 0, "kept": 0, "freed_bytes": 0}

    cutoff_seconds = time.time() - (max_days * 86400)
    deleted_count = 0
    kept_count = 0
    freed_bytes = 0

    print(f"[LOG_CLEANUP] Scanning {LOG_DIR} for telemetry logs older than {max_days} days...")

    for log_file in LOG_DIR.glob("telemetry-*.jsonl"):
        try:
            mtime = log_file.stat().st_mtime
            size = log_file.stat().st_size
            if mtime < cutoff_seconds:
                log_file.unlink()
                deleted_count += 1
                freed_bytes += size
                print(f"  [DELETED] {log_file.name} ({size / (1024*1024):.2f} MB)")
            else:
                kept_count += 1
                print(f"  [KEPT] {log_file.name} ({size / (1024*1024):.2f} MB)")
        except Exception as e:
            print(f"  [ERROR] Failed processing {log_file.name}: {e}")

    result = {
        "deleted": deleted_count,
        "kept": kept_count,
        "freed_bytes": freed_bytes,
        "freed_mb": round(freed_bytes / (1024 * 1024), 2)
    }

    print(f"[LOG_CLEANUP] Complete. Deleted {deleted_count} files ({result['freed_mb']} MB freed). Kept {kept_count} active logs.")
    return result

def main():
    parser = argparse.ArgumentParser(description="Tadpole OS Telemetry Log Retention Tool")
    parser.add_argument("--days", type=int, default=7, help="Number of days of logs to retain (default: 7)")
    args = parser.parse_args()

    prune_telemetry_logs(args.days)
    sys.exit(0)

if __name__ == "__main__":
    main()
