#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / test_openrouter_payload_live

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[-]`
- **Witness Tests**: none declared
"""

import sys
import time
import json
import urllib.request
import urllib.error
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

test_payload = {
    "model": "stealth/ox-alpha",
    "messages": [
        {"role": "system", "content": "You are a test validator for Tadpole OS. Return a simple greeting and confirm system online."},
        {"role": "user", "content": "Status check"}
    ],
    "temperature": 0.1,
    "max_tokens": 100
}

req = urllib.request.Request(
    "https://openrouter.ai/api/v1/chat/completions",
    data=json.dumps(test_payload).encode("utf-8"),
    headers=headers
)

print("Attempting request to OpenRouter with automatic 429 backoff retry loop...")
for attempt in range(1, 6):
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            raw = resp.read().decode("utf-8")
            data = json.loads(raw)
            print(f"[+] Attempt {attempt}: HTTP {resp.status} OK")
            print(f"[+] Model ID Returned: {data.get('model')}")
            print(f"[+] Provider: {data.get('provider')}")
            
            choice = data.get("choices", [{}])[0]
            msg = choice.get("message", {})
            content = msg.get("content")
            reasoning = msg.get("reasoning")
            
            print(f"[+] content:   {repr(content)}")
            print(f"[+] reasoning: {repr(reasoning)}")
            
            effective = content or reasoning or ""
            print(f"[+] Output Ingested: '{effective}'")
            print(f"[+] Finish reason: {choice.get('finish_reason')}")
            print(f"[+] Usage: {data.get('usage')}")
            break
    except urllib.error.HTTPError as e:
        if e.code == 429:
            print(f"[-] Attempt {attempt}: Received HTTP 429 (Rate limited). Backing off for 2.5s...")
            time.sleep(2.5)
        else:
            print(f"[-] HTTP Error {e.code}: {e.read().decode('utf-8')}")
            break
    except Exception as e:
        print(f"[-] Error: {e}")
        break
