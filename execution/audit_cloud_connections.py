#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / audit_cloud_connections
- **Primary Entrypoints**: `analyze_logs`, `audit_database_agents`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[-]`, `[0]`, `[:8]`, `[1]`, `[3]`, `[4]`
- **Witness Tests**: none declared
"""

import sys
import json
import sqlite3
from pathlib import Path
from collections import defaultdict, Counter

# Windows UTF-8 stdout
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
DB_PATH = ROOT / "data" / "tadpole.db"
LOGS_DIR = ROOT / "data" / "logs"

def analyze_logs():
    print("=" * 80)
    print("📡 AUDITING TELEMETRY LOGS FOR CLOUD OUTBOUND CONNECTIONS")
    print("=" * 80)

    log_files = sorted(LOGS_DIR.glob("telemetry-*.jsonl"))
    if not log_files:
        print("[-] No telemetry log files found in data/logs/")
        return

    latest_log = log_files[-1]
    print(f"[+] Inspecting latest telemetry log: {latest_log.name}")

    lines = latest_log.read_text(encoding="utf-8", errors="ignore").splitlines()
    print(f"[+] Total records in log: {len(lines)}")

    agent_spans = defaultdict(list)
    cloud_events = []
    connectivity_checks = []
    provider_calls = Counter()
    agent_providers = defaultdict(set)
    agent_tasks = defaultdict(list)

    for idx, line in enumerate(lines):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
            t = record.get("type")

            if t == "trace:span":
                span = record.get("span", {})
                name = span.get("name", "")
                agent_id = span.get("agent_id")
                attrs = span.get("attributes", {})
                uri = attrs.get("uri", "")

                if "infra_providers::test" in name or "/test" in uri:
                    cloud_events.append({"idx": idx, "type": "provider_test", "span": span})
                elif "AgentExecution" in name:
                    agent_spans[agent_id].append(span)

            elif t == "agent:status":
                a_id = record.get("agent_id")
                task = record.get("current_task", "")
                status = record.get("status")
                agent_tasks[a_id].append((status, task))

            elif t == "agent:reasoning_step":
                a_id = record.get("agent_id")
                step = record.get("step") or {}
                model = step.get("model", "unknown")
                provider_calls[f"Agent {a_id} -> Model: {model}"] += 1

        except Exception:
            pass

    print("\n--- Agent Executions in Telemetry Log ---")
    for a_id, spans in agent_spans.items():
        print(f"  • Agent ID: {a_id} -> {len(spans)} execution span(s)")

    print("\n--- Model Reasoning Invocations Recorded ---")
    if provider_calls:
        for k, v in provider_calls.items():
            print(f"  • {k} (Calls: {v})")
    else:
        print("  (No direct reasoning step records found)")

    return latest_log

def audit_database_agents():
    print("\n" + "=" * 80)
    print("🤖 AUDITING ALL AGENTS IN SQLITE (tadpole.db) FOR CLOUD PROVIDERS")
    print("=" * 80)

    if not DB_PATH.exists():
        print("[-] tadpole.db not found")
        return

    conn = sqlite3.connect(str(DB_PATH))
    c = conn.cursor()

    c.execute("SELECT id, name, role, provider, model_id, model_2, model_3, active_model_slot FROM agents;")
    rows = c.fetchall()

    cloud_agents = []
    local_agents = []
    provider_breakdown = Counter()
    model_breakdown = Counter()

    for r in rows:
        a_id, name, role, provider, m1, m2, m3, slot = r
        provider_str = (provider or "unknown").lower()
        provider_breakdown[provider_str] += 1
        model_breakdown[m1] += 1

        # Check if provider is external/cloud
        if provider_str in ["openai", "anthropic", "google", "gemini", "groq", "deepseek", "openrouter", "ollama-cloud", "fireworks", "together", "mistral", "xai"]:
            cloud_agents.append({
                "id": a_id,
                "name": name,
                "role": role,
                "provider": provider,
                "model": m1,
                "slot": slot
            })
        else:
            local_agents.append({
                "id": a_id,
                "name": name,
                "role": role,
                "provider": provider,
                "model": m1
            })

    print(f"\n[+] Total Agents in Database: {len(rows)}")
    print(f"    • Cloud / External Provider Agents: {len(cloud_agents)}")
    print(f"    • Local Provider Agents (Ollama):   {len(local_agents)}")

    print("\n[+] Provider Breakdown:")
    for p, count in provider_breakdown.most_common():
        print(f"    • {p:15s}: {count:3d} agent(s)")

    print("\n[+] Primary Model Breakdown:")
    for m, count in model_breakdown.most_common():
        print(f"    • {str(m):25s}: {count:3d} agent(s)")

    print("\n--- Agents Configured with Cloud Providers ---")
    for a in cloud_agents:
        is_ox = "ox-alpha" in str(a["model"]) or "oxalpha" in str(a["model"])
        flag = " [OX-ALPHA]" if is_ox else " ⚠️ [NON-OX-ALPHA CLOUD AGENT]"
        print(f"  Agent {a['id']:25s} ({a['name']:25s}) | Provider: {a['provider']:12s} | Model: {a['model']:22s}{flag}")

    # Check active missions
    print("\n--- Active Missions in SQLite ---")
    c.execute("SELECT id, agent_id, title, status, created_at FROM mission_history WHERE status='active';")
    active_missions = c.fetchall()
    print(f"[+] Total Active Missions: {len(active_missions)}")
    for m in active_missions:
        print(f"  Mission {m[0][:8]}... -> Agent ID: {m[1]} | Status: {m[3]} | Created: {m[4]}")

    conn.close()

def main():
    analyze_logs()
    audit_database_agents()
    print("\n" + "=" * 80)
    print("✅ Cloud connection audit complete.")
    print("=" * 80)

if __name__ == "__main__":
    main()
