#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Quality:Verification

### AI Assist Note
**ADG Symbol Gate Wrapper**: Delegates to the high-fidelity Rust AST-based 
`graph_query` validate binary, maintaining backward compatibility for python-based 
callers and pre-commit hooks.

### 🔍 Debugging & Observability
- **Failure Path**: Failed execution of cargo run or Rust compilation issues.
- **Trace Scope**: `execution::symbol_gate`
"""

import os
import subprocess
import sys
from pathlib import Path

def main():
    # Parse CLI arguments to see if --strict or --all/--full is requested
    strict_mode = "--strict" in sys.argv or "FIX=1" in sys.argv
    full_mode = "--all" in sys.argv or "--full" in sys.argv
    
    # We find the workspace root
    current = Path(__file__).resolve().parent
    workspace_root = current.parent
    server_rs_dir = workspace_root / "server-rs"
    
    # Construct the cargo command
    cmd = ["cargo", "run", "--bin", "graph_query", "--", "validate"]
    if not full_mode and "--diff" not in sys.argv:
        cmd.append("--diff")
    if strict_mode and "--strict" not in sys.argv:
        cmd.append("--strict")
    for arg in sys.argv[1:]:
        if arg not in cmd and arg not in ["--all", "--full"]:
            cmd.append(arg)
        
    try:
        # Execute the cargo binary under server-rs folder, inheriting stdio
        result = subprocess.run(
            cmd,
            cwd=str(server_rs_dir),
            check=False
        )
        sys.exit(result.returncode)
    except Exception as e:
        print(f"[FAIL] [ADG-SG] Failed to execute graph_query validate: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

# Metadata: [symbol_gate]
