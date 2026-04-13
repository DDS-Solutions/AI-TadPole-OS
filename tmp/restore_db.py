import os
import sqlite3
import re

DB_PATH = 'd:/TadpoleOS-Dev/tadpole.db'
MIGRATIONS_DIR = 'd:/TadpoleOS-Dev/migrations'
RECOVERED_SQL = 'd:/TadpoleOS-Dev/data/recovered_data.sql'

def init_db():
    print(f"--- Initializing {DB_PATH} ---")
    if os.path.exists(DB_PATH):
        os.remove(DB_PATH) # Start fresh
    
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    
    migrations = sorted([f for f in os.listdir(MIGRATIONS_DIR) if f.endswith('.sql')])
    for m in migrations:
        print(f"Applying {m}...")
        with open(os.path.join(MIGRATIONS_DIR, m), 'r', encoding='utf-8') as f:
            sql = f.read()
            cursor.executescript(sql)
    
    conn.commit()
    print("Database initialization complete.")
    return conn

def restore_agents(conn):
    print("--- Restoring Agents ---")
    cursor = conn.cursor()
    
    # Extract INSERT statements for agents
    with open(RECOVERED_SQL, 'r', encoding='utf-8') as f:
        sql_content = f.read()
    
    # regex to find INSERT INTO agents
    inserts = re.findall(r"INSERT INTO agents VALUES\((.*?)\);", sql_content)
    
    print(f"Found {len(inserts)} agent insert statements.")
    
    # Clear agents table first
    cursor.execute("DELETE FROM agents;")
    
    count = 0
    for i, values in enumerate(inserts):
        try:
            # We use executescript for the whole line to handle the values correctly
            # But we need to make sure we don't have DELETE statements in between
            stmt = f"INSERT INTO agents VALUES({values});"
            cursor.execute(stmt)
            count += 1
        except Exception as e:
            print(f"Error inserting agent {i+1}: {e}")
            # Try to see if it's a column mismatch
            cursor.execute("PRAGMA table_info(agents)")
            columns = cursor.fetchall()
            print(f"Table columns: {[c[1] for c in columns]}")
            # If mismatch, we might need to map them manually
    
    conn.commit()
    print(f"Successfully restored {count} agents.")

if __name__ == "__main__":
    conn = init_db()
    restore_agents(conn)
    
    # Double check count
    cursor = conn.cursor()
    cursor.execute("SELECT count(*) FROM agents;")
    final_count = cursor.fetchone()[0]
    print(f"Final count in DB: {final_count}")
    
    conn.close()
