import os
import sqlite3
import re

DB_PATH = 'd:/TadpoleOS-Dev/data/tadpole.db'
RECOVERED_SQL = 'd:/TadpoleOS-Dev/data/recovered_data.sql'

def restore_agents():
    print(f"--- Restoring Agents to {DB_PATH} ---")
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    
    # Extract INSERT INTO agents statements
    with open(RECOVERED_SQL, 'r', encoding='utf-8') as f:
        sql_content = f.read()
    
    # regex to find INSERT INTO agents
    inserts = re.findall(r"INSERT INTO agents VALUES\((.*?)\);", sql_content)
    
    print(f"Found {len(inserts)} agent insert statements.")
    
    # Clear agents table first (to remove baseline seeds like Agent 1)
    cursor.execute("DELETE FROM agents;")
    
    count = 0
    for i, values in enumerate(inserts):
        try:
            # We use executescript for the whole line to handle the values correctly
            stmt = f"INSERT INTO agents VALUES({values});"
            cursor.execute(stmt)
            count += 1
        except Exception as e:
            print(f"Error inserting agent {i+1}: {e}")
    
    conn.commit()
    
    # Double check count
    cursor.execute("SELECT count(*) FROM agents;")
    final_count = cursor.fetchone()[0]
    print(f"Final count in DB: {final_count}")
    
    conn.close()

if __name__ == "__main__":
    restore_agents()
