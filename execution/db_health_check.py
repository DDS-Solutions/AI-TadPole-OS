"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[db_health_check]` in system logs.
"""

import sqlite3
import os
import json
import sys
import argparse
from typing import Dict, Any, List, Union

def print_result(check: str, status: bool, message: str) -> None:
    icon = "[OK]" if status else "[FAIL]"
    print(f"{icon} [{check}] {message}")

def check_health(db_path: str) -> Dict[str, Any]:
    if not os.path.exists(db_path):
        return {"status": "error", "message": f"Database not found at {db_path}"}
    
    try:
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        
        # Check tables
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table';")
        tables = [row[0] for row in cursor.fetchall()]
        
        # Check mission count as a sample
        cursor.execute("SELECT COUNT(*) FROM mission_history;")
        mission_count = cursor.fetchone()[0]
        
        # Check agent count
        cursor.execute("SELECT COUNT(*) FROM agents;")
        agent_count = cursor.fetchone()[0]
        
        report = {
            "status": "healthy",
            "database": db_path,
            "table_count": len(tables),
            "mission_count": mission_count,
            "agent_count": agent_count,
            "tables": tables
        }
        conn.close()
        return report
    except Exception as e:
        return {"status": "unhealthy", "error": str(e)}

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Tadpole Database Health Check")
    parser.add_argument("--db", type=str, default=r"D:\TadpoleOS-Dev\tadpole.db", help="Path to database")
    parser.add_argument("--output", type=str, help="Path to output log file")
    args = parser.parse_args()

    report = check_health(args.db)
    output_json = json.dumps(report, indent=2)
    
    if args.output:
        try:
            with open(args.output, "w") as f:
                f.write(output_json)
            print_result("DB-HEALTH", True, f"Health report saved to {args.output}")
        except Exception as e:
            print_result("DB-HEALTH", False, f"Error saving to {args.output}: {e}")
            sys.exit(1)
    else:
        print(output_json)
        if report.get("status") == "healthy":
            print_result("DB-HEALTH", True, "Database is healthy")
        else:
            print_result("DB-HEALTH", False, "Database is unhealthy")

# Metadata: [db_health_check]
