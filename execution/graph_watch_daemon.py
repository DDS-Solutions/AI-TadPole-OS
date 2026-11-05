#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / graph_watch_daemon
- **Primary Entrypoints**: `reindex_graph`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

# [graph] Graph Watch Daemon implementation
"""
Graph Watch Daemon (Pillar 3)
------------------------------
Monitors source file changes (.rs, .ts, .tsx) and triggers incremental symbol graph re-indexing.
"""

import sys
import time
import subprocess
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent

def reindex_graph():
    """Triggers cargo run --bin graph_query -- audit-context."""
    print(f"[{time.strftime('%H:%M:%S')}] 🔄 [Graph Watch] File change detected. Re-indexing symbol graph...")
    cmd = [
        "cargo", "run", "--manifest-path", "server-rs/Cargo.toml",
        "--bin", "graph_query", "--", "audit-context"
    ]
    try:
        subprocess.run(cmd, cwd=WORKSPACE_ROOT, check=True)
        print(f"[{time.strftime('%H:%M:%S')}] ✅ [Graph Watch] Symbol graph synchronized.")
    except Exception as e:
        print(f"[{time.strftime('%H:%M:%S')}] ❌ [Graph Watch] Re-index failed: {e}")

def main():
    print("==================================================")
    print("      GRAPH INTELLIGENCE - HOT-RELOAD DAEMON      ")
    print("==================================================")
    print("Monitoring: server-rs/src/ and src/ for changes...")
    print("Press Ctrl+C to stop watcher.")
    print("--------------------------------------------------")

    # Initial reindex
    reindex_graph()

    last_mtime = {}
    watch_dirs = [WORKSPACE_ROOT / "server-rs" / "src", WORKSPACE_ROOT / "src"]

    try:
        while True:
            changed = False
            for d in watch_dirs:
                if not d.exists():
                    continue
                for p in d.rglob("*"):
                    if p.is_file() and p.suffix in [".rs", ".ts", ".tsx", ".js", ".jsx"]:
                        try:
                            mtime = p.stat().st_mtime
                            if p in last_mtime and last_mtime[p] != mtime:
                                changed = True
                            last_mtime[p] = mtime
                        except OSError:
                            pass

            if changed:
                reindex_graph()

            time.sleep(3)
    except KeyboardInterrupt:
        print("\n[+] Graph watch daemon stopped.")

if __name__ == "__main__":
    main()
