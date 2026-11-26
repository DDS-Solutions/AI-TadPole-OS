#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / set_openrouter_solution1
- **Primary Entrypoints**: `update_env`, `update_schema`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
env_file = ROOT / ".env"
env_example = ROOT / ".env.example"
env_schema = ROOT / ".env.schema"

def update_env(p: Path):
    if not p.exists():
        return
    lines = p.read_text(encoding="utf-8").splitlines()
    new_lines = []
    found = False
    for line in lines:
        if line.startswith("OPENROUTER_DEFAULT_MODEL="):
            new_lines.append('OPENROUTER_DEFAULT_MODEL="google/gemini-2.5-flash"')
            found = True
        else:
            new_lines.append(line)
    if not found:
        new_lines.append('OPENROUTER_DEFAULT_MODEL="google/gemini-2.5-flash"')
    p.write_text("\n".join(new_lines) + "\n", encoding="utf-8")
    print(f"[+] Set OPENROUTER_DEFAULT_MODEL in {p.name}")

def update_schema(p: Path):
    if not p.exists():
        return
    content = p.read_text(encoding="utf-8")
    content = content.replace(
        "@type=string @default=stealth/ox-alpha",
        "@type=string @default=google/gemini-2.5-flash"
    )
    p.write_text(content, encoding="utf-8")
    print(f"[+] Updated schema default in {p.name}")

update_env(env_file)
update_env(env_example)
update_schema(env_schema)
