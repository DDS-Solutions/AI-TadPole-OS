#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / debug_agent_model

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[0]`, `[1]`, `[5]`, `[2]`, `[3]`, `[4]`, `[6]`
- **Witness Tests**: none declared
"""

import sqlite3
from pathlib import Path
from collections import Counter

ROOT = Path(__file__).resolve().parent.parent
DB_PATH = ROOT / "data" / "tadpole.db"

conn = sqlite3.connect(str(DB_PATH))
c = conn.cursor()

c.execute("SELECT id, name, model_id, model_2, model_3, provider, active_model_slot FROM agents;")
rows = c.fetchall()

providers = Counter()
models = Counter()

print(f"Total Agents in DB: {len(rows)}")
for r in rows:
    providers[r[5]] += 1
    models[r[2]] += 1
    if r[0] in ['1', '2', '99', 'system_architect']:
        print(f"Agent {r[0]} ({r[1]}): provider={r[5]}, model_id={r[2]}, model_2={r[3]}, model_3={r[4]}, active_slot={r[6]}")

print("\nProviders Distribution:", dict(providers))
print("Primary Models Distribution:", dict(models))

conn.close()
