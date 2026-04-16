import sqlite3
import json

db_path = "data/tadpole.db"

def check_agents():
    try:
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT id, name, status, active_mission FROM agents")
        rows = cursor.fetchall()
        print("--- Agents and Missions ---")
        for row in rows:
            print(f"ID: {row[0]}, Name: {row[1]}, Status: {row[2]}, Mission: {row[3]}")
        conn.close()
    except Exception as e:
        print(f"Error: {row if 'row' in locals() else e}")

if __name__ == "__main__":
    check_agents()
