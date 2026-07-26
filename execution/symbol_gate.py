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
    # Parse CLI arguments to see if --strict is requested
    strict_mode = "--strict" in sys.argv or "FIX=1" in sys.argv
    
    # We find the workspace root
    current = Path(__file__).resolve().parent
    workspace_root = current.parent
    server_rs_dir = workspace_root / "server-rs"
    
    # Construct the cargo command
    cmd = ["cargo", "run", "--bin", "graph_query", "--", "validate"]
    if strict_mode:
        cmd.append("--strict")
        
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
