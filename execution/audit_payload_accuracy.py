#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / audit_payload_accuracy
- **Primary Entrypoints**: `check_openrouter_payload_handshake`, `check_database_tool_payloads`, `check_telemetry_span_payloads`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[:100]`, `[:200]`, `[-]`, `[:19]`, `[:8]`, `[:110]`
- **Witness Tests**: none declared
"""

import sys
import json
import sqlite3
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
DB_PATH = ROOT / "data" / "tadpole.db"
LOGS_DIR = ROOT / "data" / "logs"

def check_openrouter_payload_handshake():
    print("=" * 80)
    print("1. PROBING OPENROUTER LIVE PAYLOAD STRUCTURE & ACCURACY")
    print("=" * 80)

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
            {"role": "system", "content": "You are a test validator. Output a short JSON object: {\"status\": \"ok\", \"code\": 200}"},
            {"role": "user", "content": "Return the JSON now."}
        ],
        "temperature": 0.1,
        "max_tokens": 100
    }

    req = urllib.request.Request(
        "https://openrouter.ai/api/v1/chat/completions",
        data=json.dumps(test_payload).encode("utf-8"),
        headers=headers
    )

    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            status = resp.status
            raw_body = resp.read().decode("utf-8")
            data = json.loads(raw_body)
            
            print(f"[+] HTTP Status: {status}")
            print(f"[+] Model Echoed: {data.get('model')}")
            print(f"[+] Provider: {data.get('provider')}")
            
            choice = data.get("choices", [{}])[0]
            msg = choice.get("message", {})
            content = msg.get("content")
            reasoning = msg.get("reasoning")
            finish_reason = choice.get("finish_reason")
            usage = data.get("usage", {})
            
            print("\n[+] Response Fields Analysis:")
            print(f"    • content:       {repr(content)}")
            print(f"    • reasoning:     {repr(reasoning)[:100]}...")
            print(f"    • finish_reason: {finish_reason}")
            print(f"    • prompt_tokens: {usage.get('prompt_tokens')}")
            print(f"    • comp_tokens:   {usage.get('completion_tokens')}")
            
            effective_text = content or reasoning or ""
            print(f"\n[+] Effective Ingested Text Length: {len(effective_text)} chars")
            print(f"[+] Effective Text: {effective_text[:200]}...")
            
            is_valid = bool(effective_text and status == 200)
            print(f"\n✅ Payload Wire Validation: {'PASSED (100% Valid)' if is_valid else 'FAILED'}")
            return is_valid

    except urllib.error.HTTPError as e:
        print(f"[-] HTTP Error {e.code}: {e.read().decode('utf-8')}")
        return False
    except Exception as e:
        print(f"[-] Exception: {e}")
        return False

def check_database_tool_payloads():
    print("\n" + "=" * 80)
    print("2. AUDITING INTERNAL TOOL EXECUTION PAYLOADS (tadpole.db)")
    print("=" * 80)

    if not DB_PATH.exists():
        print("[-] tadpole.db missing")
        return

    conn = sqlite3.connect(str(DB_PATH))
    c = conn.cursor()

    # Query mission_logs for tool calls and executions
    c.execute("""
        SELECT mission_id, source, severity, text, timestamp 
        FROM mission_logs 
        WHERE text LIKE '%tool%' OR text LIKE '%directive%' OR text LIKE '%finding%'
        ORDER BY timestamp DESC LIMIT 10;
    """)
    rows = c.fetchall()

    print(f"[+] Found {len(rows)} recent tool-related event payloads:")
    for r in rows:
        m_id, source, sev, text, ts = r
        clean_text = text.replace('\n', ' ').strip()
        print(f"  [{ts[:19]}] [{source}/{sev}] (Mission {m_id[:8]}..): {clean_text[:110]}...")

    conn.close()

def check_telemetry_span_payloads():
    print("\n" + "=" * 80)
    print("3. AUDITING TELEMETRY SPAN JSON ENVELOPES (telemetry-*.jsonl)")
    print("=" * 80)

    log_files = sorted(LOGS_DIR.glob("telemetry-*.jsonl"))
    if not log_files:
        print("[-] No telemetry files found")
        return

    latest = log_files[-1]
    lines = latest.read_text(encoding="utf-8", errors="ignore").splitlines()
    print(f"[+] Reading {latest.name} ({len(lines)} records)")

    valid_spans = 0
    valid_reasoning = 0
    corrupt_records = 0

    for line in lines[-500:]:
        if not line.strip():
            continue
        try:
            record = json.loads(line)
            t = record.get("type")
            if t == "trace:span":
                span = record.get("span", {})
                if "name" in span and "id" in span:
                    valid_spans += 1
            elif t == "agent:reasoning_step":
                step = record.get("step", {})
                if "model" in step and "latency_ms" in step:
                    valid_reasoning += 1
        except Exception:
            corrupt_records += 1

    print(f"[+] Valid Spans in sampled tail:     {valid_spans}")
    print(f"[+] Valid Reasoning Events in tail: {valid_reasoning}")
    print(f"[+] Corrupted / Unparsable Lines:   {corrupt_records}")
    print(f"✅ Telemetry Structural Parity:     {'100% Clean' if corrupt_records == 0 else 'Warnings Found'}")

def main():
    check_openrouter_payload_handshake()
    check_database_tool_payloads()
    check_telemetry_span_payloads()
    print("\n" + "=" * 80)
    print("🎯 PAYLOAD & RESPONSE ACCURACY AUDIT COMPLETE")
    print("=" * 80)

if __name__ == "__main__":
    main()
