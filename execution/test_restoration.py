"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**CREATE TABLE agents (**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[test_restoration]` in system logs.
"""

import unittest
import sqlite3
import json
import os
import subprocess
import sys

class TestRestoreAgents(unittest.TestCase):
    def setUp(self):
        self.db_path = "test_restore.db"
        self.json_path = "test_agents.json"
        
        # 1. Create a dummy database with the 'agents' table
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute("""
            CREATE TABLE agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                department TEXT NOT NULL,
                description TEXT NOT NULL,
                model_id TEXT,
                status TEXT NOT NULL,
                metadata TEXT NOT NULL,
                skills TEXT,
                workflows TEXT,
                mcp_tools TEXT,
                provider TEXT,
                requires_oversight BOOLEAN DEFAULT 0
            )
        """)
        conn.commit()
        conn.close()

    def tearDown(self):
        if os.path.exists(self.db_path):
            os.remove(self.db_path)
        if os.path.exists(self.json_path):
            os.remove(self.json_path)

    def run_restore(self, src, dest):
        # Run the script via subprocess to test it as a cli tool
        cmd = [sys.executable, "execution/restore_agents.py", src, dest]
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result

    def test_restore_single_agent(self):
        agent = {
            "id": "test-bot",
            "name": "Test Bot",
            "role": "Tester",
            "department": "QA",
            "description": "Just a test",
            "model": "gpt-4",
            "status": "idle",
            "requires_oversight": True
        }
        with open(self.json_path, "w") as f:
            json.dump(agent, f)
            
        res = self.run_restore(self.json_path, self.db_path)
        self.assertEqual(res.returncode, 0)
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT id, name, requires_oversight FROM agents WHERE id='test-bot'")
        row = cursor.fetchone()
        conn.close()
        
        self.assertIsNotNone(row)
        self.assertEqual(row[0], "test-bot")
        self.assertEqual(row[1], "Test Bot")
        self.assertEqual(row[2], 1) # Boolean in SQLite is 0/1

    def test_restore_batch_agents(self):
        agents = [
            {"id": "bot-1", "name": "Bot 1"},
            {"id": "bot-2", "name": "Bot 2"}
        ]
        with open(self.json_path, "w") as f:
            json.dump(agents, f)
            
        res = self.run_restore(self.json_path, self.db_path)
        self.assertEqual(res.returncode, 0)
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM agents")
        count = cursor.fetchone()[0]
        conn.close()
        
        self.assertEqual(count, 2)

if __name__ == "__main__":
    unittest.main()

# Metadata: [test_restoration]
