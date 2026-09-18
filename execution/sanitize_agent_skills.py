#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / sanitize_agent_skills
- **Primary Entrypoints**: `parse_and_clean_skills`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import json
import sqlite3
from pathlib import Path

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
DB_PATH = ROOT / "data" / "tadpole.db"

def clean_token(token: str) -> str:
    return token.replace("\\", "").replace('"', "").strip()

def parse_and_clean_skills(raw_val: str | None) -> str:
    if not raw_val or not raw_val.strip() or raw_val.strip() == "null":
        return "[]"
    
    trimmed = raw_val.strip()
    
    # Check if already valid JSON list
    try:
        parsed = json.loads(trimmed)
        if isinstance(parsed, list):
            clean_list = [clean_token(str(item)) for item in parsed if clean_token(str(item))]
            return json.dumps(clean_list)
    except Exception:
        pass
    
    # Handle legacy unescaped/pseudo-JSON strings
    cleaned = (
        trimmed.replace(r"\[", "")
        .replace(r"\]", "")
        .replace("[", "")
        .replace("]", "")
        .replace(r"\,", ",")
        .replace(r"\\", "")
        .replace("\\", "")
        .replace('"', "")
    )
    
    items = [clean_token(part) for part in cleaned.split(",") if clean_token(part)]
    return json.dumps(items)

def main():
    if not DB_PATH.exists():
        print(f"Database not found at {DB_PATH}")
        sys.exit(1)
        
    conn = sqlite3.connect(str(DB_PATH))
    cursor = conn.cursor()
    
    cursor.execute("SELECT id, name, skills FROM agents;")
    rows = cursor.fetchall()
    
    updated_count = 0
    print(f"Inspecting {len(rows)} agents in database...")
    
    for agent_id, name, skills in rows:
        cleaned_json = parse_and_clean_skills(skills)
        if skills != cleaned_json:
            print(f"  [+] Sanitizing Agent '{agent_id}' ({name}):")
            print(f"      Before: {skills!r}")
            print(f"      After:  {cleaned_json}")
            cursor.execute(
                "UPDATE agents SET skills = ? WHERE id = ?;",
                (cleaned_json, agent_id)
            )
            updated_count += 1
            
    conn.commit()
    conn.close()
    
    print(f"\n[Sanitization Complete] Successfully updated {updated_count} agents.")

if __name__ == "__main__":
    main()
