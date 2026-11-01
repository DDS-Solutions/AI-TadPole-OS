#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / check_env_keys

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
env_path = ROOT / ".env"

if env_path.exists():
    for line in env_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#") and "=" in line:
            key, val = line.split("=", 1)
            clean_val = val.strip().strip('"').strip("'")
            print(f"  {key:32s} -> Configured: {bool(clean_val)}")
else:
    print(".env not found")
