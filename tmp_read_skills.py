import sqlite3
import json

db_path = "c:\\Users\\Home Office_PC\\.gemini\\antigravity\\playground\\tadpole-os\\tadpole.db"
try:
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM skills WHERE name='issue_alpha_directive'")
    rows = cursor.fetchall()
    
    if not rows:
        print("Not found! Let's list all skills:")
        cursor.execute("SELECT id, name FROM skills")
        for row in cursor.fetchall():
            print(dict(row))
    else:
        for row in rows:
            print(dict(row))
except Exception as e:
    print(f"Error: {e}")
