import sqlite3
import os
import json
import sys
import argparse

def check_health(db_path):
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
            print(f"Health report saved to {args.output}")
        except Exception as e:
            print(f"Error saving to {args.output}: {e}")
            sys.exit(1)
    else:
        print(output_json)
