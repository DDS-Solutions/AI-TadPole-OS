#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / set_default_model_self
- **Primary Entrypoints**: `update_file`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

"""
Sets OPENROUTER_DEFAULT_MODEL="stealth/ox-alpha" in .env and .env.example
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
env_file = ROOT / ".env"
env_example = ROOT / ".env.example"

def update_file(p: Path):
    if not p.exists():
        return
    lines = p.read_text(encoding="utf-8").splitlines()
    new_lines = []
    found = False
    for line in lines:
        if line.startswith("OPENROUTER_DEFAULT_MODEL="):
            new_lines.append('OPENROUTER_DEFAULT_MODEL="stealth/ox-alpha"')
            found = True
        else:
            new_lines.append(line)
    if not found:
        new_lines.append('OPENROUTER_DEFAULT_MODEL="stealth/ox-alpha"')
    p.write_text("\n".join(new_lines) + "\n", encoding="utf-8")
    print(f"[+] Updated OPENROUTER_DEFAULT_MODEL in {p.name} to stealth/ox-alpha")

update_file(env_file)
update_file(env_example)
