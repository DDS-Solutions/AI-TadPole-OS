#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / update_env_openrouter
- **Primary Entrypoints**: `update_file`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

"""
Safely appends OPENROUTER_DEFAULT_MODEL to .env and .env.example
without printing any secret credentials.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
env_file = ROOT / ".env"
env_example = ROOT / ".env.example"

def update_file(p: Path):
    if not p.exists():
        return
    content = p.read_text(encoding="utf-8")
    if "OPENROUTER_DEFAULT_MODEL" not in content:
        addition = "\n# Default OpenRouter model for sovereign stealth/ox-alpha agents\nOPENROUTER_DEFAULT_MODEL=\"google/gemini-2.5-flash\"\n"
        p.write_text(content.rstrip() + addition, encoding="utf-8")
        print(f"[+] Added OPENROUTER_DEFAULT_MODEL to {p.name}")
    else:
        print(f"[*] OPENROUTER_DEFAULT_MODEL already present in {p.name}")

update_file(env_file)
update_file(env_example)
