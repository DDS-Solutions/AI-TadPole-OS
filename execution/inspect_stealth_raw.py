#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / inspect_stealth_raw

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
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
    "X-Title": "TadpoleOS",
    "Content-Type": "application/json"
}

payload = json.dumps({
    "model": "stealth/ox-alpha",
    "messages": [{"role": "user", "content": "hello"}],
    "max_tokens": 15
}).encode("utf-8")

req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=payload, headers=headers)
try:
    with urllib.request.urlopen(req, timeout=10) as r:
        raw = r.read().decode("utf-8")
        print("HTTP Status:", r.status)
        print("Raw OpenRouter Response:\n", json.dumps(json.loads(raw), indent=2))
except urllib.error.HTTPError as e:
    print(f"HTTP Error {e.code}:", e.read().decode("utf-8"))
except Exception as e:
    print("Exception:", e)
