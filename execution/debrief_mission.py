"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / debrief_mission
- **Primary Entrypoints**: `resolve_database_path`, `get_mission_data`, `extract_wisdom`, `commit_wisdom`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[OK]`, `[WARNING]`, `[--commit]`, `[PARTIAL]`, `[FAIL]`
- **Witness Tests**: none declared
"""

"""
# 🧠 Agent 99: Mission Debriefing & Wisdom Extraction
**Layer**: Execution (Deterministic)
**Capability**: Semantic Wisdom Synthesis
**Standard**: ECC-MEM (Enhanced Contextual Clarity - Memory)

Usage:
    python execution/debrief_mission.py <MISSION_ID> [--commit]
"""

import sqlite3
import os
import sys
import json
import requests
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Optional
from dotenv import load_dotenv

# Audit Chain import for ALCOA+ tamper-evident logging
try:
    from execution.audit_chain import AuditChain, format_merkle_trail_markdown
except ImportError:
    from audit_chain import AuditChain, format_merkle_trail_markdown


# Load env vars
load_dotenv()

# Constants
ROOT = Path(__file__).resolve().parent.parent


def resolve_database_path() -> Path:
    """Resolve SQLite URLs relative to the repository instead of a developer machine."""
    configured = os.getenv("DATABASE_URL", "sqlite://data/tadpole.db")
    raw_path = configured.removeprefix("sqlite://").removeprefix("sqlite:")
    path = Path(raw_path)
    return path if path.is_absolute() else ROOT / path


DB_PATH = resolve_database_path()
MEMORY_PATH = ROOT / "directives" / "LONG_TERM_MEMORY.md"
FAULT_PATH = ROOT / "directives" / "FAULT_REGISTRY.md"
GOOGLE_API_KEY = os.getenv("GOOGLE_API_KEY")

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def get_mission_data(mission_id):
    """Fetch goal and logs for a mission"""
    if not DB_PATH.exists():
        print(f"ERROR: Database not found at {DB_PATH}")
        sys.exit(1)

    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    # Get goal
    cursor.execute("SELECT id, title, status FROM mission_history WHERE id = ?", (mission_id,))
    mission = cursor.fetchone()
    if not mission:
        print(f"ERROR: Mission {mission_id} not found")
        sys.exit(1)

    # Get logs
    cursor.execute("""
        SELECT source, text, severity, timestamp 
        FROM mission_logs 
        WHERE mission_id = ? 
        ORDER BY timestamp ASC
    """, (mission_id,))
    logs = cursor.fetchall()
    
    conn.close()
    return mission, logs

def extract_wisdom(mission, logs):
    """Call LLM to extract technical wisdom"""
    mission_id, goal, status = mission
    
    # Format logs for context
    log_text = "\n".join([f"[{l[0]}]: {l[1]}" for l in logs])
    
    prompt = f"""
Analyze the following Mission Logs from Tadpole OS and extract "Instructional Wisdom".
Focus on architectural discoveries, recurring bug patterns, or best practices learned during this mission.

MISSION GOAL: {goal}
FINAL STATUS: {status}

### LOGS ###
{log_text}

### OUTPUT FORMAT ###
Return a JSON object with exactly two keys:
1. "wisdom": A string containing the bulleted list of "Key Learnings" (Format: "- **Topic**: [Insight] (Verification: [How we proved it])").
2. "faults": A list of objects representing captured failures (Format: [{{"topic": "...", "input": "...", "bad_output": "...", "desired_output": "..."}}]).

If no faults are found, return an empty list for "faults".
Do not include conversational filler or markdown code blocks outside the JSON.
"""

    # Use Groq as the fallback/primary because Gemini key is invalid
    GROQ_API_KEY = os.getenv("GROQ_API_KEY")
    GROQ_BASE_URL = os.getenv("GROQ_BASE_URL", "https://api.groq.com/openai/v1")

    url = f"{GROQ_BASE_URL}/chat/completions"
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {GROQ_API_KEY}"
    }
    payload = {
        "model": "llama-3.3-70b-versatile",
        "messages": [
            {"role": "system", "content": "You are a mission analyst for Tadpole OS. Analyze logs and extract technical wisdom."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1
    }

    try:
        response = requests.post(url, headers=headers, json=payload, timeout=60)
        if not response.ok:
            print(f"ERROR: API returned {response.status_code}: {response.text}")
            return None
            
        data = response.json()
        raw_content = data['choices'][0]['message']['content'].strip()
        
        # Strip markdown code blocks if present
        if raw_content.startswith("```json"):
            raw_content = raw_content[7:-3].strip()
        elif raw_content.startswith("```"):
            raw_content = raw_content[3:-3].strip()
            
        return json.loads(raw_content, strict=False)
    except Exception as e:
        print(f"ERROR: LLM Analysis or Parsing failed: {e}")
        return None

def commit_wisdom(wisdom, logs=None):
    """Append wisdom and SHA-256 Merkle trail to LONG_TERM_MEMORY.md and SQLite agent_audit_logs"""
    if not MEMORY_PATH.exists():
        print(f"ERROR: {MEMORY_PATH} not found")
        return False

    # Compute tamper-evident hash chain over mission logs if provided
    merkle_markdown = ""
    chain_entries = []
    if logs:
        chain = AuditChain()
        for l in logs:
            source, text = l[0], l[1]
            chain.append(source, text)
        chain_entries = chain.entries
        merkle_markdown = format_merkle_trail_markdown(chain_entries)
        
        # Dual-Write: Persist Merkle chain to SQLite agent_audit_logs if database exists
        try:
            DB_PATH.parent.mkdir(parents=True, exist_ok=True)
            conn = sqlite3.connect(DB_PATH)
            cursor = conn.cursor()
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS agent_audit_logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    prev_hash TEXT NOT NULL,
                    hash TEXT NOT NULL,
                    timestamp REAL NOT NULL,
                    agent_id TEXT NOT NULL,
                    content TEXT NOT NULL
                )
            """)
            for entry in chain_entries:
                cursor.execute("""
                    INSERT INTO agent_audit_logs (prev_hash, hash, timestamp, agent_id, content)
                    VALUES (?, ?, ?, ?, ?)
                """, (entry['prev_hash'], entry['hash'], entry['timestamp'], entry['agent_id'], entry['content']))
            conn.commit()
            conn.close()
            print(f"[OK] Persisted {len(chain_entries)} tamper-evident audit records to SQLite agent_audit_logs")
        except Exception as e:
            print(f"[WARNING] Could not write audit chain to SQLite: {e}")

    with open(MEMORY_PATH, "r", encoding="utf-8") as f:
        content = f.read()

    # Find the "Key Learnings" section
    if "## Key Learnings" not in content:
        print("ERROR: Could not find '## Key Learnings' section in memory file")
        return False

    # Insert wisdom and Merkle trail after header
    header = "## Key Learnings"
    new_entry = f"\n{wisdom}\n{merkle_markdown}\n"
    
    updated_content = content.replace(header, f"{header}{new_entry}")
    
    with open(MEMORY_PATH, "w", encoding="utf-8") as f:
        f.write(updated_content)
    
    return True

def commit_faults(mission, faults):
    """Sync faults to FAULT_REGISTRY.md and SQLite"""
    mission_id, goal, status = mission
    
    if not faults:
        return True

    # 1. Update SQLite
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()
        for f in faults:
            fault_id = f"fault_{mission_id}_{datetime.now().microsecond}"
            cursor.execute("""
                INSERT INTO fault_registry (id, mission_id, agent_id, topic, input, bad_output, desired_output)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (fault_id, mission_id, "agent_99", f.get('topic', 'Logic'), f.get('input'), f.get('bad_output'), f.get('desired_output')))
        conn.commit()
        conn.close()
    except Exception as e:
        print(f"ERROR: Failed to update fault_registry table: {e}")

    # 2. Update FAULT_REGISTRY.md
    if not FAULT_PATH.exists():
        return False

    with open(FAULT_PATH, "r", encoding="utf-8") as f:
        content = f.read()

    marker = "<!-- FAULT_ENTRIES_START -->"
    if marker not in content:
        return False

    new_entries = ""
    for f in faults:
        timestamp = datetime.now().strftime("%Y-%m-%d")
        new_entries += f"| {mission_id} | {mission_id} | agent_99 | {f.get('topic')} | unresolved |\n"
        # Add a detailed section below the table or just the table row
    
    updated_content = content.replace(marker, f"{marker}\n{new_entries}")

    with open(FAULT_PATH, "w", encoding="utf-8") as f:
        f.write(updated_content)

    return True

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: debrief_mission.py <MISSION_ID> [--commit]")
        sys.exit(1)

    mission_id = sys.argv[1]
    commit = "--commit" in sys.argv

    print(f"📡 [Agent 99] Starting debrief for mission: {mission_id}")
    
    mission, logs = get_mission_data(mission_id)
    print(f"📝 Logs fetched: {len(logs)} entries")
    
    analysis = extract_wisdom(mission, logs)
    
    if analysis:
        wisdom = analysis.get("wisdom")
        faults = analysis.get("faults", [])
        
        print("\n🧠 --- EXTRACTED WISDOM ---")
        print(wisdom)
        print(f"Found {len(faults)} faults.")
        print("---------------------------\n")
        
        if commit:
            success_wisdom = commit_wisdom(wisdom, logs)
            success_faults = commit_faults(mission, faults)
            
            if success_wisdom and success_faults:
                print(f"[OK] Learnings and Faults committed.")
            else:
                print("[PARTIAL] One or more commit operations failed.")
    else:
        print("[FAIL] No wisdom extracted")
