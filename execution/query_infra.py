#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / query_infra
- **Primary Entrypoints**: `get`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import os
import json
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

token = ""
env_file = ROOT / ".env"
if env_file.exists():
    with open(env_file, "r", encoding="utf-8") as f:
        for line in f:
            if line.startswith("NEURAL_TOKEN="):
                token = line.split("=", 1)[1].strip().strip('"').strip("'")

headers = {"Authorization": f"Bearer {token}"} if token else {}

def get(endpoint):
    url = f"http://127.0.0.1:8000/v1/{endpoint}"
    req = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=5) as r:
            return json.loads(r.read().decode())
    except Exception as e:
        return {"error": str(e)}

print("=== NODES (/v1/infra/nodes) ===")
nodes = get("infra/nodes")
print(json.dumps(nodes, indent=2))

print("\n=== MODELS LIST ===")
models = get("infra/models")
if isinstance(models, list):
    for m in models:
        print(f"ID: {m.get('id'):40s} | Name: {m.get('name'):30s} | Provider: {m.get('provider_id')}")
