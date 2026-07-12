"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**🧬 evaluate_annealing: Fault-to-Directive Transformation**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[evaluate_annealing]` in system logs.
"""

#!/usr/bin/env python3
"""
# 🧬 evaluate_annealing: Fault-to-Directive Transformation
**Layer**: Execution (Deterministic / Proposal)
**Capability**: Sovereign Self-Hardening

Usage:
    python execution/evaluate_annealing.py [--fault-id <ID>]
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

# Load env vars
load_dotenv()

# Constants
DB_PATH = Path(os.getenv("DATABASE_URL", "D:/TadpoleOS-Dev/data/tadpole.db").replace("sqlite:", ""))
DIRECTIVES_PATH = Path("directives")
GROQ_API_KEY = os.getenv("GROQ_API_KEY")
GROQ_BASE_URL = os.getenv("GROQ_BASE_URL", "https://api.groq.com/openai/v1")

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def get_unresolved_faults(fault_id=None):
    """Fetch faults that need annealing"""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    
    if fault_id:
        cursor.execute("SELECT * FROM fault_registry WHERE id = ?", (fault_id,))
    else:
        cursor.execute("SELECT * FROM fault_registry WHERE status = 'unresolved'")
    
    columns = [column[0] for column in cursor.description]
    faults = [dict(zip(columns, row)) for row in cursor.fetchall()]
    conn.close()
    return faults

def propose_annealing(fault):
    """Call LLM to propose a directive update based on a fault triad"""
    
    # Identify best directive match (heuristic or semantic)
    # For now, we'll ask the model to suggest which directive to update OR create a new logic guard
    
    prompt = f"""
You are the Tadpole Engine Self-Hardening Logic. Your task is to analyze a "Fault Triad" and propose an update to a system directive to prevent this error from recurring.

### FAULT TRIAD ###
TOPIC: {fault['topic']}
INPUT: {fault['input']}
BAD OUTPUT: {fault['bad_output']}
DESIRED OUTPUT: {fault['desired_output']}

### TASK ###
1. Identify if this should go into an existing directive (e.g., GEMINI.md, Rust optimization, API handling) or a new "Security Guard" block.
2. Draft a concise "Logic Guard" or "Instructional Override" that would have prevented the Bad Output.
3. Return a JSON proposal.

### OUTPUT FORMAT ###
Return ONLY a JSON object:
{{
    "directive_name": "filename.md",
    "description": "Short explanation of the fix",
    "proposed_change": "Exact markdown snippet to add or modify",
    "rationale": "Why this fixes the fault"
}}
"""

    url = f"{GROQ_BASE_URL}/chat/completions"
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {GROQ_API_KEY}"
    }
    payload = {
        "model": "llama-3.3-70b-versatile",
        "messages": [
            {"role": "system", "content": "You are a professional systems architect specializing in AI Alignment and SOP hardening."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1
    }

    try:
        response = requests.post(url, headers=headers, json=payload, timeout=60)
        if not response.ok:
            return None
            
        data = response.json()
        raw_content = data['choices'][0]['message']['content'].strip()
        if raw_content.startswith("```json"):
            raw_content = raw_content[7:-3].strip()
        elif raw_content.startswith("```"):
            raw_content = raw_content[3:-3].strip()
        return json.loads(raw_content, strict=False)
    except Exception as e:
        print(f"ERROR: Analysis failed: {e}")
        return None

def create_proposal(fault_id, proposal_data):
    """Write the proposal to capability_proposals table"""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    
    proposal_id = f"anneal_{fault_id}_{datetime.now().strftime('%H%M%S')}"
    name = f"Annealing: {proposal_data['directive_name']}"
    description = proposal_data['description']
    content = json.dumps(proposal_data)
    
    cursor.execute("""
        INSERT INTO capability_proposals (id, name, description, cap_type, content, agent_id, status)
        VALUES (?, ?, ?, ?, ?, ?, ?)
    """, (proposal_id, name, description, 'anneal', content, 'agent_99', 'pending'))
    
    conn.commit()
    conn.close()
    return proposal_id

if __name__ == "__main__":
    fault_id = None
    if "--fault-id" in sys.argv:
        idx = sys.argv.index("--fault-id")
        if idx + 1 < len(sys.argv):
            fault_id = sys.argv[idx + 1]

    print(f"[SEARCH] Scanning for unresolved faults...")
    faults = get_unresolved_faults(fault_id)
    
    if not faults:
        print("[OK] No unresolved faults found.")
        sys.exit(0)

    for fault in faults:
        print(f"[EVAL] Evaluating Fault: {fault['id']} ({fault['topic']})")
        proposal = propose_annealing(fault)
        if proposal:
            pid = create_proposal(fault['id'], proposal)
            print(f"[OK] Proposal created: {pid}")
        else:
            print(f"[FAIL] Failed to generate proposal for {fault['id']}")

# Metadata: [evaluate_annealing]
