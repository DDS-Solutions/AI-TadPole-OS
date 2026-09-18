#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / test_stealth_direct

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[SUCCESS]`, `[ERROR]`
- **Witness Tests**: none declared
"""

import sys
import urllib.request
import urllib.error
import json
from pathlib import Path

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

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

variants = [
    "stealth/ox-alpha",
    "stealth/oxalpha",
    "ox-alpha",
    "oxalpha"
]

print("Testing stealth model slugs on OpenRouter API:")
for model in variants:
    payload = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": "hello"}],
        "max_tokens": 10
    }).encode("utf-8")
    
    req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=payload, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=10) as r:
            res = json.loads(r.read().decode())
            reply = res.get("choices", [{}])[0].get("message", {}).get("content", "").strip()
            print(f"  [SUCCESS] {model:25s} -> HTTP 200 OK | reply: '{reply}'")
    except urllib.error.HTTPError as e:
        err_body = e.read().decode("utf-8")
        print(f"  [ERROR]   {model:25s} -> HTTP {e.code}: {err_body}")
    except Exception as e:
        print(f"  [ERROR]   {model:25s} -> {e}")
