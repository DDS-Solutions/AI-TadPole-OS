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

def restore_table(cursor, sql_content, table_name):
    print(f"--- Restoring {table_name} ---")
    
    # regex to find INSERT INTO {table_name} VALUES(...)
    # Note: we need to handle multi-line if needed, but these seem to be one-line.
    # Also handle unistr(...) for mission_logs
    inserts = re.findall(r"INSERT INTO " + table_name + r" VALUES\((.*?)\);", sql_content)
    
    print(f"Found {len(inserts)} {table_name} insert statements.")
    
    # We don't delete here unless it's agents, which we already cleared in the main script logic if we want.
    # But for 100% restoration, we should clear all target tables.
    cursor.execute(f"DELETE FROM {table_name};")
    
    count = 0
    for i, values in enumerate(inserts):
        try:
            # Check for unistr(...) - SQLite doesn't support unistr directly.
            # We might need to convert unistr('...\u000a...') to regular string or just use the raw string.
            # SQLite supports hex strings but not unistr.
            if "unistr(" in values:
                # Basic conversion for unistr to something SQLite understands
                # Actually, if we use cursor.execute(stmt), SQLite will fail on unistr
                # Let's replace unistr(...) with the internal string
                updated_values = re.sub(r"unistr\('(.*?)'\)", r"'\1'", values)
                # Note: \u000a etc may not be converted.
            else:
                updated_values = values
                
            stmt = f"INSERT INTO {table_name} VALUES({updated_values});"
            cursor.execute(stmt)
            count += 1
        except Exception as e:
            print(f"Error inserting {table_name} record {i+1}: {e}")
    
    print(f"Successfully restored {count} {table_name} records.")

if __name__ == "__main__":
    conn = init_db()
    cursor = conn.cursor()
    
    with open(RECOVERED_SQL, 'r', encoding='utf-8') as f:
        sql_content = f.read()
    
    tables = ['agents', 'mission_history', 'mission_logs', 'swarm_context', 'agent_quotas', 'audit_trail']
    
    for table in tables:
        restore_table(cursor, sql_content, table)
    
    conn.commit()
    
    # Double check agent count
    cursor.execute("SELECT count(*) FROM agents;")
    final_count = cursor.fetchone()[0]
    print(f"Final Agent Count: {final_count}")
    
    conn.close()
