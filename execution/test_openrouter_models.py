#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / test_openrouter_models

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[-]`
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

candidate_models = [
    "google/gemini-2.5-flash",
    "google/gemini-flash-1.5",
    "google/gemini-2.0-flash",
    "google/gemini-3.7-flash",
    "google/gemini-3.5-flash-lite",
    "meta-llama/llama-3.3-70b-instruct",
    "openai/gpt-4o-mini",
    "deepseek/deepseek-chat",
    "qwen/qwen-2.5-72b-instruct",
    "mistralai/mistral-7b-instruct",
]

print("Testing candidate models on OpenRouter:")
working_models = []

for model in candidate_models:
    payload = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 5
    }).encode("utf-8")
    
    req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=payload, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=8) as r:
            res = json.loads(r.read().decode())
            reply = res.get("choices", [{}])[0].get("message", {}).get("content", "").strip()
            print(f"  [+] {model:35s} -> OK (reply: '{reply}')")
            working_models.append(model)
    except urllib.error.HTTPError as e:
        err_body = e.read().decode("utf-8")
        print(f"  [-] {model:35s} -> HTTP {e.code}: {err_body}")
    except Exception as e:
        print(f"  [-] {model:35s} -> {e}")

print("\nWorking Models on OpenRouter:", working_models)
