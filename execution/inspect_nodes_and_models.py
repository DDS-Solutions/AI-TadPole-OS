#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / inspect_nodes_and_models
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sqlite3
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DB_PATH = ROOT / "data" / "tadpole.db"

def main():
    conn = sqlite3.connect(str(DB_PATH))
    c = conn.cursor()
    c.execute("SELECT name FROM sqlite_master WHERE type='table';")
    tables = [r[0] for r in c.fetchall()]
    print("Tables:", tables)

    for t in tables:
        if any(k in t.lower() for k in ["node", "model", "provider", "infra", "bunker"]):
            print(f"\n--- Table: {t} ---")
            c.execute(f"SELECT * FROM {t} LIMIT 10;")
            cols = [desc[0] for desc in c.description]
            print("Columns:", cols)
            for row in c.fetchall():
                print(dict(zip(cols, row)))

    conn.close()

if __name__ == "__main__":
    main()
