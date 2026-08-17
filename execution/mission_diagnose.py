"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Tadpole OS Mission Diagnostics Tool**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[mission_diagnose]` in system logs.
"""

#!/usr/bin/env python3
"""
Tadpole OS Mission Diagnostics Tool
"""

import sqlite3
import os
import sys
import json
from pathlib import Path
from datetime import datetime

# Load env vars or use defaults
ROOT = Path(__file__).resolve().parent.parent
DATABASE_VALUE = os.getenv("DATABASE_URL", "sqlite://data/tadpole.db")
DATABASE_PATH = Path(DATABASE_VALUE.removeprefix("sqlite://").removeprefix("sqlite:"))
DB_PATH = DATABASE_PATH if DATABASE_PATH.is_absolute() else ROOT / DATABASE_PATH
LOGS_DIR = ROOT / "data" / "logs"

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def get_latest_mission():
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("SELECT id, title, status, agent_id, created_at FROM mission_history ORDER BY created_at DESC LIMIT 1")
    mission = cursor.fetchone()
    conn.close()
    return mission

def get_mission_by_id(mission_id):
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("SELECT id, title, status, agent_id, created_at FROM mission_history WHERE id = ?", (mission_id,))
    mission = cursor.fetchone()
    conn.close()
    return mission

def get_mission_logs(mission_id):
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("""
        SELECT source, text, severity, timestamp 
        FROM mission_logs 
        WHERE mission_id = ? 
        ORDER BY timestamp ASC
    """, (mission_id,))
    logs = cursor.fetchall()
    conn.close()
    return logs

def parse_telemetry(mission_id, mission_date_str):
    log_file = LOGS_DIR / f"telemetry-{mission_date_str}.jsonl"
    if not log_file.exists():
        return [], None
        
    spans = {}
    events = []
    
    with open(log_file, "r", encoding="utf-8") as f:
        for line in f:
            if not line.strip():
                continue
            try:
                data = json.loads(line)
                t_type = data.get("type")
                if t_type == "trace:span":
                    span = data["span"]
                    s_id = span["id"]
                    s_mission = span.get("mission_id") or span.get("attributes", {}).get("mission_id")
                    if s_mission == mission_id or s_mission == f"Some(\"{mission_id}\")":
                        spans[s_id] = span
                        events.append({
                            "time": span["start_time"],
                            "type": "start",
                            "name": span["name"],
                            "span_id": s_id,
                            "agent_id": span.get("agent_id") or span.get("attributes", {}).get("agent_id"),
                            "details": span
                        })
                elif t_type == "trace:span_update":
                    s_id = data["span_id"]
                    if s_id in spans:
                        update = data["update"]
                        events.append({
                            "time": update["end_time"],
                            "type": "end",
                            "name": spans[s_id]["name"],
                            "span_id": s_id,
                            "status": update["status"],
                            "details": update
                        })
            except Exception:
                pass
                
    events.sort(key=lambda x: x["time"])
    return events, spans

def main():
    mission_id = sys.argv[1] if len(sys.argv) > 1 else None
    
    if not DB_PATH.exists():
        print(f"❌ Error: Database not found at {DB_PATH}")
        sys.exit(1)
        
    if mission_id:
        mission = get_mission_by_id(mission_id)
    else:
        mission = get_latest_mission()
        
    if not mission:
        print("❌ Error: No mission found in database.")
        sys.exit(1)
        
    m_id, m_title, m_status, m_agent_id, m_created_at = mission
    
    print("=" * 70)
    print(f"🐸 TADPOLE OS MISSION DIAGNOSTICS")
    print("=" * 70)
    print(f"Mission ID: {m_id}")
    print(f"Goal/Title: {m_title}")
    print(f"Agent ID:   {m_agent_id}")
    print(f"Status:     {m_status.upper()}")
    print(f"Started At: {m_created_at}")
    print("=" * 70)
    
    # 1. Fetch DB Logs
    db_logs = get_mission_logs(m_id)
    print(f"\n📝 Database Logs ({len(db_logs)} entries):")
    for log in db_logs:
        timestamp, source, severity, text = log[3], log[0], log[2], log[1]
        sev_color = "🔴" if severity == "fatal" or severity == "error" else "🟢" if severity == "success" else "⚪"
        print(f"  [{timestamp}] {sev_color} [{source}] [{severity.upper()}] {text}")
        
    # 2. Correlate with Telemetry Spans
    try:
        date_part = m_created_at.split("T")[0]
        telemetry_events, spans = parse_telemetry(m_id, date_part)
    except Exception as e:
        print(f"\n⚠️ Failed to parse telemetry log: {e}")
        telemetry_events, spans = [], None
        
    if telemetry_events:
        print(f"\n🔭 Telemetry Execution Trace ({len(telemetry_events)} span transitions):")
        open_spans = {}
        for ev in telemetry_events:
            dt = datetime.fromtimestamp(ev["time"] / 1000.0).isoformat()
            if ev["type"] == "start":
                open_spans[ev["span_id"]] = ev
                agent_info = f" [{ev['agent_id']}]" if ev.get("agent_id") else ""
                print(f"  [{dt}] START: {ev['name']}{agent_info} (ID: {ev['span_id']})")
            else:
                if ev["span_id"] in open_spans:
                    start_ev = open_spans.pop(ev["span_id"])
                    duration = ev["time"] - start_ev["time"]
                    print(f"  [{dt}] END:   {ev['name']} (ID: {ev['span_id']}) -> {ev['status']} ({duration}ms)")
                else:
                    print(f"  [{dt}] END:   {ev['name']} (ID: {ev['span_id']}) -> {ev['status']}")
                    
        # Check for unclosed spans
        if open_spans:
            print("\n🚨 CRITICAL ANALYSIS: ABRUPT HALT DETECTED")
            print("The following trace spans started execution but never completed:")
            for s_id, s_ev in open_spans.items():
                agent_info = f" [{s_ev['agent_id']}]" if s_ev.get("agent_id") else ""
                print(f"  ⚠️  Span: {s_ev['name']}{agent_info} (ID: {s_id})")
            print("\n💡 Signature: The engine likely crashed or panicked during this span's execution.")
            print("   Verify the crash using the crash reconciler or by checking sidecar_panic.log.")
    else:
        print("\n⚠️ No telemetry spans found matching this mission date/ID.")

if __name__ == "__main__":
    main()

# Metadata: [mission_diagnose]
