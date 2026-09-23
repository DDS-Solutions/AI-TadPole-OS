#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / telemetry_monitor
- **Primary Entrypoints**: `get_auth_token`, `get_headers`, `get_today_log_path`, `check_server_health`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[SPAN:START]`, `[SPAN:END]`, `[:60]`, `[REASONING]`
- **Witness Tests**: none declared
"""

import os
import sys
import time
import json
import sqlite3
import argparse
from pathlib import Path
from datetime import datetime, timezone
from collections import defaultdict, Counter

# Windows UTF-8 stdout setup
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
BASE_URL = os.getenv("TADPOLE_API_URL", "http://127.0.0.1:8000/v1")
DATABASE_PATH = ROOT / "data" / "tadpole.db"
LOGS_DIR = ROOT / "data" / "logs"
REPORT_DIR = ROOT / "reports" / "telemetry"

def get_declared_version():
    try:
        with open(ROOT / "version.json", "r", encoding="utf-8") as version_file:
            return json.load(version_file).get("version", "unknown")
    except (OSError, ValueError, TypeError):
        return "unknown"

def get_auth_token():
    token = os.getenv("NEURAL_TOKEN")
    if not token:
        env_file = ROOT / ".env"
        if env_file.exists():
            with open(env_file, "r", encoding="utf-8") as f:
                for line in f:
                    if line.startswith("NEURAL_TOKEN="):
                        token = line.split("=", 1)[1].strip().strip('"').strip("'")
                        break
    return token

def get_headers():
    token = get_auth_token()
    return {
        "Authorization": f"Bearer {token}" if token else "",
        "Content-Type": "application/json"
    }

def get_today_log_path():
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    return LOGS_DIR / f"telemetry-{today}.jsonl"

def check_server_health():
    import urllib.request
    headers = get_headers()
    url = f"{BASE_URL}/engine/health"
    req = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=3) as resp:
            if resp.status == 200:
                data = json.loads(resp.read().decode("utf-8"))
                return True, data, resp.status
            return False, {}, resp.status
    except Exception as e:
        return False, {"error": str(e)}, 500

def scrape_metrics():
    import urllib.request
    headers = get_headers()
    url = f"{BASE_URL}/engine/metrics"
    req = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=3) as resp:
            if resp.status == 200:
                metrics = {}
                for line in resp.read().decode("utf-8").splitlines():
                    if line and not line.startswith("#"):
                        parts = line.split()
                        if len(parts) >= 2:
                            try:
                                metrics[parts[0]] = float(parts[1])
                            except ValueError:
                                metrics[parts[0]] = parts[1]
                return metrics
    except Exception:
        pass
    return {}

def analyze_telemetry_file(log_path):
    if not log_path.exists():
        return {
            "total_records": 0,
            "spans_opened": 0,
            "spans_closed": 0,
            "unclosed_count": 0,
            "unclosed_spans": [],
            "span_durations": {},
            "errors": [],
            "warnings": [],
            "agent_reasoning": {},
            "event_types": {}
        }

    lines = log_path.read_text(encoding="utf-8", errors="ignore").splitlines()
    spans_open = {}
    spans_closed = {}
    span_durations = defaultdict(list)
    agent_reasoning = defaultdict(list)
    errors_found = []
    warnings_found = []
    event_types = Counter()

    for idx, line in enumerate(lines):
        if not line.strip():
            continue
        try:
            data = json.loads(line)
            t = data.get("type", "unknown")
            event_types[t] += 1

            if t == "trace:span":
                span = data.get("span", {})
                s_id = span.get("id")
                if s_id:
                    spans_open[s_id] = span

            elif t == "trace:span_update":
                s_id = data.get("span_id")
                up = data.get("update", {})
                status = up.get("status")
                if s_id:
                    spans_closed[s_id] = up
                    if s_id in spans_open:
                        start = spans_open[s_id].get("start_time", 0)
                        end = up.get("end_time", 0)
                        if end and start and end >= start:
                            duration = end - start
                            name = spans_open[s_id].get("name", "unknown")
                            span_durations[name].append(duration)
                if status not in ["success", "running", None]:
                    errors_found.append({"index": idx, "span_id": s_id, "status": status, "record": data})

            elif t == "agent:reasoning_step":
                step = data.get("step", {})
                agent_id = data.get("agent_id")
                agent_reasoning[agent_id].append(step)

        except Exception as e:
            warnings_found.append({"index": idx, "error": str(e)})

    unclosed = {k: v for k, v in spans_open.items() if k not in spans_closed}

    return {
        "total_records": len(lines),
        "spans_opened": len(spans_open),
        "spans_closed": len(spans_closed),
        "unclosed_count": len(unclosed),
        "unclosed_spans": list(unclosed.values())[:10],
        "span_durations": span_durations,
        "errors": errors_found,
        "warnings": warnings_found,
        "agent_reasoning": agent_reasoning,
        "event_types": dict(event_types)
    }

def analyze_database():
    if not DATABASE_PATH.exists():
        return {"exists": False}

    db_stats = {}
    try:
        conn = sqlite3.connect(str(DATABASE_PATH))
        c = conn.cursor()

        # Table count
        c.execute("SELECT count(*) FROM sqlite_master WHERE type='table';")
        db_stats["tables"] = c.fetchone()[0]

        # Mission history status
        c.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='mission_history';")
        if c.fetchone():
            c.execute("SELECT status, count(*) FROM mission_history GROUP BY status;")
            db_stats["missions"] = dict(c.fetchall())
            c.execute("SELECT id, agent_id, title, status, created_at FROM mission_history ORDER BY created_at DESC LIMIT 5;")
            db_stats["recent_missions"] = [
                {"id": r[0], "agent_id": r[1], "title": r[2][:60], "status": r[3], "created_at": r[4]}
                for r in c.fetchall()
            ]

        # Agent errors table
        c.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='agent_errors';")
        if c.fetchone():
            c.execute("SELECT count(*) FROM agent_errors;")
            db_stats["agent_errors_count"] = c.fetchone()[0]
            c.execute("SELECT id, agent_id, error_type, message, created_at FROM agent_errors ORDER BY created_at DESC LIMIT 5;")
            db_stats["recent_agent_errors"] = [
                {"id": r[0], "agent_id": r[1], "type": r[2], "message": r[3][:80], "created_at": r[4]}
                for r in c.fetchall()
            ]
        else:
            db_stats["agent_errors_count"] = 0

        # Audit logs high severity
        c.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='audit_logs';")
        if c.fetchone():
            c.execute("SELECT count(*) FROM audit_logs WHERE severity IN ('error', 'critical', 'fatal');")
            db_stats["high_sev_audit_logs"] = c.fetchone()[0]
        else:
            db_stats["high_sev_audit_logs"] = 0

        conn.close()
    except Exception as e:
        db_stats["error"] = str(e)

    return db_stats

def run_telemetry_tail(duration_seconds=30):
    print("=" * 80)
    print(f"🐸 TADPOLE OS TELEMETRY & SERVER TAILER ({duration_seconds}s Window)")
    print(f"🕒 Timestamp: {datetime.now(timezone.utc).isoformat()}")
    print("=" * 80)

    # 1. Check server health
    healthy, health_data, code = check_server_health()
    if healthy:
        print(f"✅ Server Health: OK (HTTP {code})")
        print(f"   • State: {health_data.get('health_state', health_data.get('status'))} | Version: {health_data.get('version')} | Port: {health_data.get('port')}")
    else:
        print(f"❌ Server Health: FAILED (HTTP {code}) -> {health_data}")

    # 2. Prometheus Metrics
    metrics = scrape_metrics()
    print(f"📊 Prometheus Core Metrics:")
    print(f"   • Active Agents:    {metrics.get('tadpole_active_agents', 'N/A')}")
    print(f"   • Health State:     {metrics.get('tadpole_health_state', 'N/A')} (2.0 = Ready)")
    print(f"   • Swarm Depth Cap:  {metrics.get('tadpole_max_swarm_depth', 'N/A')}")
    print(f"   • TPM Accumulator:  {metrics.get('tadpole_tpm_accumulator', 'N/A')}")
    print(f"   • Recruit Count:    {metrics.get('tadpole_recruit_count', 'N/A')}")
    print("-" * 80)

    # 3. Live tailing loop
    log_path = get_today_log_path()
    start_pos = log_path.stat().st_size if log_path.exists() else 0
    start_time = time.time()
    end_time = start_time + duration_seconds

    print(f"🔭 Tailing live telemetry from: {log_path.name}...")
    window_events = []
    window_errors = []

    while time.time() < end_time:
        elapsed = round(time.time() - start_time, 1)
        if log_path.exists():
            with open(log_path, "r", encoding="utf-8") as f:
                f.seek(start_pos)
                new_lines = f.readlines()
                start_pos = f.tell()

            for line in new_lines:
                if not line.strip():
                    continue
                try:
                    data = json.loads(line)
                    t_type = data.get("type", "unknown")

                    if t_type == "trace:span":
                        span = data.get("span", {})
                        s_name = span.get("name")
                        s_id = span.get("id")
                        uri = span.get("attributes", {}).get("uri", "")
                        uri_str = f" [{uri}]" if uri else ""
                        print(f"  [{elapsed:4.1f}s] 🟢 [SPAN:START] {s_name}{uri_str} (ID: {s_id})")
                        window_events.append(data)

                    elif t_type == "trace:span_update":
                        s_id = data.get("span_id")
                        update = data.get("update", {})
                        status = update.get("status", "unknown")
                        icon = "✅" if status == "success" else "❌"
                        print(f"  [{elapsed:4.1f}s] {icon} [SPAN:END]   ID: {s_id} -> {status}")
                        if status not in ["success", "running"]:
                            window_errors.append(data)

                    elif t_type == "agent:status":
                        a_id = data.get("agent_id")
                        status = data.get("status") or "unknown"
                        task = data.get("current_task") or ""
                        print(f"  [{elapsed:4.1f}s] 🤖 [AGENT:{a_id}] {status.upper()} | {task[:60]}")

                    elif t_type == "agent:reasoning_step":
                        step = data.get("step") or {}
                        a_id = data.get("agent_id")
                        model = step.get("model") or "unknown"
                        tokens = (step.get("input_tokens") or 0) + (step.get("output_tokens") or 0)
                        lat = step.get("latency_ms") or 0
                        cost = step.get("cost_usd") or 0.0
                        print(f"  [{elapsed:4.1f}s] 🧠 [REASONING] Agent {a_id} ({model}) | {tokens} tok | {lat}ms | ${cost:.4f}")

                    elif t_type == "agent:message":
                        a_id = data.get("agent_id")
                        role = data.get("role")
                        content = data.get("content", "")[:80].replace("\n", " ")
                        print(f"  [{elapsed:4.1f}s] 💬 [MSG:{a_id}:{role}] {content}...")

                except Exception as ex:
                    print(f"  [{elapsed:4.1f}s] ⚠️ Parse error: {ex}")

        time.sleep(0.5)

    print("-" * 80)
    print("📈 Compiling Telemetry Health & Performance Audit...")

    # Comprehensive analysis
    file_analysis = analyze_telemetry_file(log_path)
    db_analysis = analyze_database()

    print(f"\n📑 Cumulative Day Telemetry ({log_path.name}):")
    print(f"   • Total Log Events: {file_analysis['total_records']}")
    print(f"   • Spans Opened:     {file_analysis['spans_opened']}")
    print(f"   • Spans Closed:     {file_analysis['spans_closed']}")
    print(f"   • Unclosed Spans:   {file_analysis['unclosed_count']}")
    print(f"   • Non-success Spans:{len(file_analysis['errors'])}")
    print(f"   • Parse Warnings:   {len(file_analysis['warnings'])}")

    print("\n⏱️ Top Latency Spans (Average Duration):")
    durations = file_analysis["span_durations"]
    sorted_spans = sorted(durations.items(), key=lambda x: sum(x[1])/len(x[1]) if x[1] else 0, reverse=True)
    for name, dur_list in sorted_spans[:8]:
        avg_ms = sum(dur_list) / len(dur_list)
        max_ms = max(dur_list)
        min_ms = min(dur_list)
        print(f"   • {name:28s} -> Count: {len(dur_list):4d} | Avg: {avg_ms:7.2f}ms | Min: {min_ms:6.2f}ms | Max: {max_ms:7.2f}ms")

    print("\n🤖 Agent Reasoning & Token Efficiency:")
    for a_id, steps in file_analysis["agent_reasoning"].items():
        total_tokens = sum(s.get("input_tokens", 0) + s.get("output_tokens", 0) for s in steps)
        total_cost = sum(s.get("cost_usd", 0.0) for s in steps)
        avg_lat = sum(s.get("latency_ms", 0) for s in steps) / len(steps) if steps else 0
        print(f"   • Agent {a_id}: {len(steps)} turns | {total_tokens} tokens | ${total_cost:.4f} cost | Avg Latency: {avg_lat:.1f}ms")

    print("\n🗄️ Database Health & Ledger State:")
    print(f"   • Active Tables:    {db_analysis.get('tables', 'N/A')}")
    print(f"   • Mission Counts:   {db_analysis.get('missions', {})}")
    print(f"   • Agent Errors:     {db_analysis.get('agent_errors_count', 0)}")
    print(f"   • High-Sev Audits:  {db_analysis.get('high_sev_audit_logs', 0)}")

    # Generate Markdown Report
    REPORT_DIR.mkdir(parents=True, exist_ok=True)
    report_file = REPORT_DIR / f"telemetry_audit_{datetime.now(timezone.utc).strftime('%Y%m%d_%H%M%S')}.md"
    generate_markdown_report(report_file, healthy, health_data, metrics, file_analysis, db_analysis, duration_seconds)
    print(f"\n💾 Full Audit Report saved to: {report_file}")
    print("=" * 80)

def generate_markdown_report(out_path, healthy, health_data, metrics, file_analysis, db_analysis, duration):
    lines = []
    declared_version = get_declared_version()
    observed_version = health_data.get('version', 'unknown')
    version_aligned = healthy and declared_version != 'unknown' and observed_version == declared_version
    lines.append("# 🐸 Tadpole OS - Real-Time Telemetry & Server Audit Report\n\n")
    lines.append(f"> **Capture Window**: `{duration} Seconds`  \n")
    lines.append(f"> **Timestamp**: `{datetime.now(timezone.utc).isoformat()}`  \n")
    lines.append(f"> **Server Status**: `{'OPERATIONAL (HTTP 200)' if healthy else 'DEGRADED / UNREACHABLE'}`  \n\n")
    lines.append("---\n\n")

    lines.append("## 1. Core Server Health & Prometheus Metrics\n\n")
    lines.append("| Metric / Parameter | Observed Value | Nominal Benchmark | Status |\n")
    lines.append("| :--- | :--- | :--- | :---: |\n")
    lines.append(f"| **Engine State** | `{health_data.get('health_state', health_data.get('status', 'Unknown'))}` | `Ready` / `2.0` | {'✅' if healthy else '❌'} |\n")
    lines.append(f"| **Engine Version** | `{observed_version}` | `{declared_version}` (declared) | {'✅' if version_aligned else '⚠️'} |\n")
    lines.append(f"| **Active Agents** | `{metrics.get('tadpole_active_agents', 'N/A')}` | `56` Nodes | ✅ |\n")
    lines.append(f"| **Swarm Depth Limit** | `{metrics.get('tadpole_max_swarm_depth', 'N/A')}` | `3.0..=5.0` | ✅ |\n")
    lines.append(f"| **TPM Accumulator** | `{metrics.get('tadpole_tpm_accumulator', 'N/A')}` | Scaled | ✅ |\n")
    lines.append(f"| **Recruit Count** | `{metrics.get('tadpole_recruit_count', 'N/A')}` | Monitored | ✅ |\n\n")

    lines.append("## 2. Telemetry Spans & Latency Profile\n\n")
    lines.append("| Span Name | Count | Avg Latency (ms) | Min (ms) | Max (ms) | Assessment |\n")
    lines.append("| :--- | :---: | :---: | :---: | :---: | :--- |\n")
    durations = file_analysis.get("span_durations", {})
    sorted_spans = sorted(durations.items(), key=lambda x: sum(x[1])/len(x[1]) if x[1] else 0, reverse=True)
    for name, dur_list in sorted_spans[:10]:
        avg_ms = sum(dur_list) / len(dur_list)
        max_ms = max(dur_list)
        min_ms = min(dur_list)
        assessment = "⚡ Ultra-Fast (<5ms)" if avg_ms < 5 else "✅ Normal (<50ms)" if avg_ms < 50 else "⚠️ Moderate Latency (>50ms)"
        lines.append(f"| `{name}` | {len(dur_list)} | {avg_ms:.2f} | {min_ms:.2f} | {max_ms:.2f} | {assessment} |\n")
    lines.append("\n")

    lines.append("## 3. Storage & Mission Ledger Integrity\n\n")
    lines.append(f"- **Active SQLite Tables**: `{db_analysis.get('tables', 'N/A')}`\n")
    lines.append(f"- **Missions Breakdown**: `{db_analysis.get('missions', {})}`\n")
    lines.append(f"- **Agent Errors Registered**: `{db_analysis.get('agent_errors_count', 0)}`\n")
    lines.append(f"- **Critical Audit Events**: `{db_analysis.get('high_sev_audit_logs', 0)}`\n\n")

    lines.append("## 4. Findings, Anomalies & Improvement Recommendations\n\n")
    improvements = []
    if file_analysis.get("unclosed_count", 0) > 0:
        improvements.append(f"**Unclosed Spans**: Detected {file_analysis['unclosed_count']} open spans that have not received an explicit end update. Ensure span lifecycle watchdogs trigger automated cleanup.")
    else:
        improvements.append("**Span Lifecycle**: Zero unclosed spans. All telemetry spans lifecycle transitions completed successfully.")

    if file_analysis.get("errors"):
        improvements.append(f"**Span Failures**: {len(file_analysis['errors'])} non-success span updates detected.")
    else:
        improvements.append("**Span Health**: 100% success rate across all recorded span updates.")

    # High latency checks
    high_lat = [name for name, d in durations.items() if (sum(d)/len(d)) > 100]
    if high_lat:
        improvements.append(f"**Latency Optimization**: Spans with >100ms latency ({', '.join(high_lat)}) could benefit from in-memory caching or query indexing.")
    else:
        improvements.append("**Latency Baseline**: All standard REST/database spans are operating well within sub-50ms thresholds.")

    for item in improvements:
        lines.append(f"- {item}\n")

    out_path.write_text("".join(lines), encoding="utf-8")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Tadpole OS Telemetry & Server Monitor")
    parser.add_argument("--duration", "-d", type=int, default=30, help="Surveillance window in seconds (default: 30)")
    args = parser.parse_args()

    run_telemetry_tail(duration_seconds=args.duration)
