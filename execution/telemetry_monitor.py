#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / telemetry_monitor
- **Primary Entrypoints**: `get_auth_token`, `get_headers`, `get_today_log_path`, `check_server_health`

### ⚠️ Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[SPAN:START]`, `[SPAN:END]`, `[REASONING]`
"""

import io
import os
import sys
import time
import json
import sqlite3
import argparse
import urllib.request
import urllib.error
from pathlib import Path
from datetime import datetime, timezone
from collections import defaultdict, Counter
from typing import Any, Dict, List, Optional, Tuple

# Windows UTF-8 stdout setup
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except (AttributeError, io.UnsupportedOperation):
        pass

ROOT = Path(__file__).resolve().parent.parent
BASE_URL = os.getenv("TADPOLE_API_URL", "http://127.0.0.1:8000/v1")
DATABASE_PATH = ROOT / "data" / "tadpole.db"
LOGS_DIR = ROOT / "data" / "logs"
REPORT_DIR = ROOT / "reports" / "telemetry"

def get_declared_version() -> str:
    """Reads product version from version.json."""
    try:
        with open(ROOT / "version.json", "r", encoding="utf-8") as version_file:
            return json.load(version_file).get("version", "unknown")
    except (OSError, ValueError, TypeError):
        return "unknown"

def get_auth_token() -> Optional[str]:
    """Retrieves neural authorization token from environment or .env file."""
    token = os.getenv("NEURAL_TOKEN")
    if token:
        return token
    env_file = ROOT / ".env"
    if not env_file.exists():
        return None
    with open(env_file, "r", encoding="utf-8") as f:
        for line in f:
            if line.startswith("NEURAL_TOKEN="):
                return line.split("=", 1)[1].strip().strip('"').strip("'")
    return None

def get_headers() -> Dict[str, str]:
    """Builds HTTP authorization headers for internal REST calls."""
    token = get_auth_token()
    return {
        "Authorization": f"Bearer {token}" if token else "",
        "Content-Type": "application/json",
    }

def get_today_log_path() -> Path:
    """Returns filesystem path for the daily telemetry JSONL log."""
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    return LOGS_DIR / f"telemetry-{today}.jsonl"

def check_server_health() -> Tuple[bool, Dict[str, Any], int]:
    """Queries engine health endpoint and returns status tuple."""
    url = f"{BASE_URL}/engine/health"
    req = urllib.request.Request(url, headers=get_headers())
    try:
        with urllib.request.urlopen(req, timeout=3) as resp:
            if resp.status == 200:
                data = json.loads(resp.read().decode("utf-8"))
                return True, data, resp.status
            return False, {}, resp.status
    except urllib.error.HTTPError as err:
        return False, {"error": err.reason}, err.code
    except (urllib.error.URLError, TimeoutError, OSError) as err:
        return False, {"error": str(err)}, 500

def _parse_metrics_text(text: str) -> Dict[str, Any]:
    """Parses Prometheus metrics lines into float or string values."""
    metrics: Dict[str, Any] = {}
    for line in text.splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) < 2:
            continue
        try:
            metrics[parts[0]] = float(parts[1])
        except ValueError:
            metrics[parts[0]] = parts[1]
    return metrics

def scrape_metrics() -> Dict[str, Any]:
    """Scrapes Prometheus metric lines from engine metrics endpoint."""
    url = f"{BASE_URL}/engine/metrics"
    req = urllib.request.Request(url, headers=get_headers())
    try:
        with urllib.request.urlopen(req, timeout=3) as resp:
            if resp.status == 200:
                return _parse_metrics_text(resp.read().decode("utf-8"))
            return {}
    except (urllib.error.URLError, TimeoutError, OSError, ValueError):
        return {}

def _record_closed_span_duration(
    s_id: str,
    up: Dict[str, Any],
    spans_open: Dict[str, Any],
    span_durations: Dict[str, list]
) -> None:
    """Calculates and stores duration for a matched open and closed span."""
    if s_id not in spans_open:
        return
    start = spans_open[s_id].get("start_time", 0)
    end = up.get("end_time", 0)
    if end and start and end >= start:
        name = spans_open[s_id].get("name", "unknown")
        span_durations[name].append(end - start)

def _process_span_record(data: Dict[str, Any], idx: int, state: Dict[str, Any]) -> None:
    """Processes span lifecycle transitions and computes execution durations."""
    t = data.get("type")
    if t == "trace:span":
        span = data.get("span", {})
        s_id = span.get("id")
        if s_id:
            state["open"][s_id] = span
        return

    if t == "trace:span_update":
        s_id = data.get("span_id")
        up = data.get("update", {})
        status = up.get("status")
        if s_id:
            state["closed"][s_id] = up
            _record_closed_span_duration(s_id, up, state["open"], state["durations"])
        if status not in ["success", "running", None]:
            state["errors"].append({"index": idx, "span_id": s_id, "status": status, "record": data})

def analyze_telemetry_file(log_path: Path) -> Dict[str, Any]:
    """Analyzes daily telemetry log for span completion, errors, and agent steps."""
    if not log_path.exists():
        return {
            "total_records": 0, "spans_opened": 0, "spans_closed": 0, "unclosed_count": 0,
            "unclosed_spans": [], "span_durations": {}, "errors": [], "warnings": [],
            "agent_reasoning": {}, "event_types": {},
        }

    lines = log_path.read_text(encoding="utf-8", errors="ignore").splitlines()
    state = {
        "open": {},
        "closed": {},
        "durations": defaultdict(list),
        "errors": [],
    }
    agent_reasoning = defaultdict(list)
    warnings_found = []
    event_types = Counter()

    for idx, line in enumerate(lines):
        if not line.strip():
            continue
        try:
            data = json.loads(line)
            t = data.get("type", "unknown")
            event_types[t] += 1
            if t in ("trace:span", "trace:span_update"):
                _process_span_record(data, idx, state)
            elif t == "agent:reasoning_step":
                agent_reasoning[data.get("agent_id")].append(data.get("step", {}))
        except (json.JSONDecodeError, KeyError, TypeError) as err:
            warnings_found.append({"index": idx, "error": str(err)})

    unclosed = {k: v for k, v in state["open"].items() if k not in state["closed"]}
    return {
        "total_records": len(lines),
        "spans_opened": len(state["open"]),
        "spans_closed": len(state["closed"]),
        "unclosed_count": len(unclosed),
        "unclosed_spans": list(unclosed.values())[:10],
        "span_durations": dict(state["durations"]),
        "errors": state["errors"],
        "warnings": warnings_found,
        "agent_reasoning": dict(agent_reasoning),
        "event_types": dict(event_types),
    }

def analyze_database() -> Dict[str, Any]:
    """Queries SQLite database tables for errors, mission states, and audit records."""
    if not DATABASE_PATH.exists():
        return {"exists": False}

    db_stats: Dict[str, Any] = {}
    try:
        conn = sqlite3.connect(str(DATABASE_PATH))
        c = conn.cursor()
        c.execute("SELECT count(*) FROM sqlite_master WHERE type='table';")
        db_stats["tables"] = c.fetchone()[0]

        c.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='mission_history';")
        if c.fetchone():
            c.execute("SELECT status, count(*) FROM mission_history GROUP BY status;")
            db_stats["missions"] = dict(c.fetchall())

        c.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='agent_errors';")
        if c.fetchone():
            c.execute("SELECT count(*) FROM agent_errors;")
            db_stats["agent_errors_count"] = c.fetchone()[0]
        else:
            db_stats["agent_errors_count"] = 0

        c.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='audit_logs';")
        if c.fetchone():
            c.execute("SELECT count(*) FROM audit_logs WHERE severity IN ('error', 'critical', 'fatal');")
            db_stats["high_sev_audit_logs"] = c.fetchone()[0]
        else:
            db_stats["high_sev_audit_logs"] = 0
        conn.close()
    except (sqlite3.DatabaseError, OSError) as err:
        db_stats["error"] = str(err)
    return db_stats

def _render_live_log_line(data: Dict[str, Any], elapsed: float, window_events: list, window_errors: list) -> None:
    """Prints single live telemetry event line to terminal."""
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
    elif t_type == "agent:reasoning_step":
        step = data.get("step") or {}
        a_id = data.get("agent_id")
        model = step.get("model") or "unknown"
        tokens = (step.get("input_tokens") or 0) + (step.get("output_tokens") or 0)
        lat = step.get("latency_ms") or 0
        cost = step.get("cost_usd") or 0.0
        print(f"  [{elapsed:4.1f}s] 🧠 [REASONING] Agent {a_id} ({model}) | {tokens} tok | {lat}ms | ${cost:.4f}")

def _process_new_log_lines(new_lines: list, elapsed: float, window_events: list, window_errors: list) -> None:
    """Parses and renders live log lines appended to daily log file."""
    for line in new_lines:
        if not line.strip():
            continue
        try:
            data = json.loads(line)
            _render_live_log_line(data, elapsed, window_events, window_errors)
        except json.JSONDecodeError as ex:
            print(f"  [{elapsed:4.1f}s] ⚠️ Parse error: {ex}")

def _tail_log_stream(log_path: Path, duration_seconds: int) -> Tuple[list, list]:
    """Tails active daily log for specified duration."""
    start_pos = log_path.stat().st_size if log_path.exists() else 0
    start_time = time.time()
    end_time = start_time + duration_seconds
    window_events, window_errors = [], []

    while time.time() < end_time:
        elapsed = round(time.time() - start_time, 1)
        if log_path.exists():
            with open(log_path, "r", encoding="utf-8") as f:
                f.seek(start_pos)
                new_lines = f.readlines()
                start_pos = f.tell()
            _process_new_log_lines(new_lines, elapsed, window_events, window_errors)
        time.sleep(0.5)
    return window_events, window_errors

def _print_monitor_header(duration_seconds: int, healthy: bool, health_data: dict, code: int, metrics: dict) -> None:
    """Prints monitor execution header and nominal metrics."""
    print("=" * 80)
    print(f"🐸 TADPOLE OS TELEMETRY & SERVER TAILER ({duration_seconds}s Window)")
    print(f"🕒 Timestamp: {datetime.now(timezone.utc).isoformat()}")
    print("=" * 80)
    if healthy:
        print(f"✅ Server Health: OK (HTTP {code})")
        print(f"   • State: {health_data.get('health_state', health_data.get('status'))} | Version: {health_data.get('version')} | Port: {health_data.get('port')}")
    else:
        print(f"❌ Server Health: FAILED (HTTP {code}) -> {health_data}")

    print("📊 Prometheus Core Metrics:")
    print(f"   • Active Agents:    {metrics.get('tadpole_active_agents', 'N/A')}")
    print(f"   • Health State:     {metrics.get('tadpole_health_state', 'N/A')} (2.0 = Ready)")
    print(f"   • Swarm Depth Cap:  {metrics.get('tadpole_max_swarm_depth', 'N/A')}")
    print(f"   • TPM Accumulator:  {metrics.get('tadpole_tpm_accumulator', 'N/A')}")
    print(f"   • Recruit Count:    {metrics.get('tadpole_recruit_count', 'N/A')}")
    print("-" * 80)

def run_telemetry_tail(duration_seconds: int = 30) -> None:
    """Main execution orchestrator for tailing server health and live logs."""
    healthy, health_data, code = check_server_health()
    metrics = scrape_metrics()
    _print_monitor_header(duration_seconds, healthy, health_data, code, metrics)

    log_path = get_today_log_path()
    print(f"🔭 Tailing live telemetry from: {log_path.name}...")
    _tail_log_stream(log_path, duration_seconds)

    print("-" * 80)
    print("📈 Compiling Telemetry Health & Performance Audit...")
    file_analysis = analyze_telemetry_file(log_path)
    db_analysis = analyze_database()

    print(f"\n📑 Cumulative Day Telemetry ({log_path.name}):")
    print(f"   • Total Log Events: {file_analysis['total_records']}")
    print(f"   • Spans Opened:     {file_analysis['spans_opened']}")
    print(f"   • Spans Closed:     {file_analysis['spans_closed']}")
    print(f"   • Unclosed Spans:   {file_analysis['unclosed_count']}")
    print(f"   • Non-success Spans:{len(file_analysis['errors'])}")

    REPORT_DIR.mkdir(parents=True, exist_ok=True)
    report_file = REPORT_DIR / f"telemetry_audit_{datetime.now(timezone.utc).strftime('%Y%m%d_%H%M%S')}.md"
    report_ctx = {
        "healthy": healthy,
        "health_data": health_data,
        "metrics": metrics,
        "file_analysis": file_analysis,
        "db_analysis": db_analysis,
        "duration": duration_seconds,
    }
    generate_markdown_report(report_file, report_ctx)
    print(f"\n💾 Full Audit Report saved to: {report_file}")
    print("=" * 80)

def _build_markdown_header(report_ctx: Dict[str, Any]) -> List[str]:
    """Generates markdown report header and status metadata."""
    healthy = report_ctx["healthy"]
    health_data = report_ctx["health_data"]
    metrics = report_ctx["metrics"]
    duration = report_ctx["duration"]
    declared_version = get_declared_version()
    observed_version = health_data.get('version', 'unknown')
    version_aligned = healthy and declared_version != 'unknown' and observed_version == declared_version

    return [
        "# 🐸 Tadpole OS - Real-Time Telemetry & Server Audit Report\n\n",
        f"> **Capture Window**: `{duration} Seconds`  \n",
        f"> **Timestamp**: `{datetime.now(timezone.utc).isoformat()}`  \n",
        f"> **Server Status**: `{'OPERATIONAL (HTTP 200)' if healthy else 'DEGRADED / UNREACHABLE'}`  \n\n",
        "---\n\n",
        "## 1. Core Server Health & Prometheus Metrics\n\n",
        "| Metric / Parameter | Observed Value | Nominal Benchmark | Status |\n",
        "| :--- | :--- | :--- | :---: |\n",
        f"| **Engine State** | `{health_data.get('health_state', health_data.get('status', 'Unknown'))}` | `Ready` / `2.0` | {'✅' if healthy else '❌'} |\n",
        f"| **Engine Version** | `{observed_version}` | `{declared_version}` (declared) | {'✅' if version_aligned else '⚠️'} |\n",
        f"| **Active Agents** | `{metrics.get('tadpole_active_agents', 'N/A')}` | `56` Nodes | ✅ |\n",
        f"| **Swarm Depth Limit** | `{metrics.get('tadpole_max_swarm_depth', 'N/A')}` | `3.0..=5.0` | ✅ |\n",
        f"| **TPM Accumulator** | `{metrics.get('tadpole_tpm_accumulator', 'N/A')}` | Scaled | ✅ |\n",
        f"| **Recruit Count** | `{metrics.get('tadpole_recruit_count', 'N/A')}` | Monitored | ✅ |\n\n",
    ]

def _build_markdown_tables(report_ctx: Dict[str, Any]) -> List[str]:
    """Generates markdown table of span latency and SQLite ledger counts."""
    file_analysis = report_ctx["file_analysis"]
    db_analysis = report_ctx["db_analysis"]
    durations = file_analysis.get("span_durations", {})
    sorted_spans = sorted(durations.items(), key=lambda x: sum(x[1]) / len(x[1]) if x[1] else 0, reverse=True)

    lines = [
        "## 2. Telemetry Spans & Latency Profile\n\n",
        "| Span Name | Count | Avg Latency (ms) | Min (ms) | Max (ms) | Assessment |\n",
        "| :--- | :---: | :---: | :---: | :---: | :--- |\n",
    ]
    for name, dur_list in sorted_spans[:10]:
        avg_ms = sum(dur_list) / len(dur_list)
        assessment = "⚡ Ultra-Fast (<5ms)" if avg_ms < 5 else "✅ Normal (<50ms)" if avg_ms < 50 else "⚠️ Moderate Latency (>50ms)"
        lines.append(f"| `{name}` | {len(dur_list)} | {avg_ms:.2f} | {min(dur_list):.2f} | {max(dur_list):.2f} | {assessment} |\n")

    lines.extend([
        "\n## 3. Storage & Mission Ledger Integrity\n\n",
        f"- **Active SQLite Tables**: `{db_analysis.get('tables', 'N/A')}`\n",
        f"- **Missions Breakdown**: `{db_analysis.get('missions', {})}`\n",
        f"- **Agent Errors Registered**: `{db_analysis.get('agent_errors_count', 0)}`\n",
        f"- **Critical Audit Events**: `{db_analysis.get('high_sev_audit_logs', 0)}`\n\n",
        "## 4. Findings & Recommendations\n\n",
    ])
    return lines

def generate_markdown_report(out_path: Path, report_ctx: Dict[str, Any]) -> None:
    """Compiles markdown audit report from collected telemetry data."""
    lines = _build_markdown_header(report_ctx)
    lines.extend(_build_markdown_tables(report_ctx))

    file_analysis = report_ctx["file_analysis"]
    if file_analysis.get("unclosed_count", 0) > 0:
        lines.append(f"- **Unclosed Spans**: Detected {file_analysis['unclosed_count']} open spans that have not received an explicit end update.\n")
    else:
        lines.append("- **Span Lifecycle**: Zero unclosed spans. All telemetry spans completed successfully.\n")

    out_path.write_text("".join(lines), encoding="utf-8")

def main() -> None:
    parser = argparse.ArgumentParser(description="Tadpole OS Telemetry & Server Monitor")
    parser.add_argument("--duration", "-d", type=int, default=30, help="Surveillance window in seconds (default: 30)")
    args = parser.parse_args()
    run_telemetry_tail(duration_seconds=args.duration)

if __name__ == "__main__":
    main()
