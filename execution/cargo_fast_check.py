#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / cargo_fast_check
- **Primary Entrypoints**: `run_fast_cargo_check`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[FAST_CHECK]`, `[FAST_CHECK_ERROR]`
- **Witness Tests**: none declared
"""

import os
import shutil
import sys
import subprocess
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
SERVER_RS_DIR = WORKSPACE_ROOT / "server-rs"

def run_fast_cargo_check() -> bool:
    """Execute cargo check in server-rs directory with sccache caching when available."""
    if not SERVER_RS_DIR.exists():
        print(f"[FAST_CHECK] server-rs directory not found at {SERVER_RS_DIR}")
        return False

    env = os.environ.copy()
    if shutil.which("sccache"):
        env["RUSTC_WRAPPER"] = "sccache"
        print("[FAST_CHECK] sccache detected; running with build cache wrapper.")

    print(f"[FAST_CHECK] Running cargo check in {SERVER_RS_DIR}...")
    try:
        res = subprocess.run(
            ["cargo", "check", "--workspace"],
            cwd=str(SERVER_RS_DIR),
            env=env,
            capture_output=True,
            text=True
        )
        if res.returncode == 0:
            print("[FAST_CHECK] Cargo check PASSED clean.")
            return True
        else:
            print("[FAST_CHECK] Cargo check FAILED:")
            print(res.stderr)
            return False
    except Exception as e:
        print(f"[FAST_CHECK_ERROR] Cargo execution failed: {e}")
        return False

def main():
    success = run_fast_cargo_check()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
