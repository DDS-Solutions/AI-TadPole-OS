import sqlite3
import json

db_path = "c:\\Users\\Home Office_PC\\.gemini\\antigravity\\playground\\tadpole-os\\tadpole.db"
try:
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()
    cursor.execute("SELECT id, name, role, skills FROM agents WHERE id='1' OR name='Agent of Nine'")
    rows = cursor.fetchall()
    for row in rows:
        print(dict(row))
    
    if not rows:
        cursor.execute("SELECT id, name, role, skills FROM agents LIMIT 10")
        for row in cursor.fetchall():
            print(dict(row))
except Exception as e:
    print(f"Error: {e}")
