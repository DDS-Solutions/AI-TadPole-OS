#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / check_openrouter

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[:15]`
- **Witness Tests**: none declared
"""

import urllib.request
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

token = ""
for line in (ROOT / ".env").read_text(encoding="utf-8").splitlines():
    if line.startswith("OPENROUTER_API_KEY="):
        token = line.split("=", 1)[1].strip().strip('"').strip("'")

headers = {
    "Authorization": f"Bearer {token}",
    "HTTP-Referer": "https://tadpole-os.local",
    "X-Title": "TadpoleOS"
}

req = urllib.request.Request("https://openrouter.ai/api/v1/models", headers=headers)
try:
    with urllib.request.urlopen(req, timeout=10) as r:
        data = json.loads(r.read().decode())
        models = [m["id"] for m in data.get("data", [])]
        print(f"Total OpenRouter models available: {len(models)}")
        print("Sample models:", [m for m in models if "gemini" in m or "llama-3.3" in m or "claude" in m][:15])
except Exception as e:
    print("Models fetch error:", e)
