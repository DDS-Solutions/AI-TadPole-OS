#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Unit Tests for Graph Harness Tools**
Verifies CLI behavior, JSON slice generation, and error handling for graph tools.

### 🔍 Debugging & Observability
- **Failure Path**: Script error or execution logic mismatch.
- **Telemetry Link**: Search `[graph]` in system logs.
"""
# [graph] Test Graph Harness implementation
"""
Unit Tests for Graph Harness Tools (Pillars 1-4)
------------------------------------------------
Verifies CLI behavior, JSON slice generation, and error handling for:
- graph_blast_guard.py
- graph_prune_anomalies.py
"""

import unittest
import json
import subprocess
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent

class TestGraphHarness(unittest.TestCase):

    def test_blast_guard_json(self):
        cmd = [
            "python", "execution/graph_blast_guard.py",
            "--path", "server-rs/src/routes/system.rs",
            "--name", "get_workspaces_status",
            "--json"
        ]
        res = subprocess.run(cmd, cwd=WORKSPACE_ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace")
        self.assertEqual(res.returncode, 0, f"blast guard failed: {res.stderr}")
        data = json.loads(res.stdout)
        self.assertEqual(data.get("target_file"), "server-rs/src/routes/system.rs")
        self.assertEqual(data.get("target_symbol"), "get_workspaces_status")
        self.assertIn("blast_radius_symbol_count", data)

    def test_prune_anomalies_dry_run(self):
        cmd = [
            "python", "execution/graph_prune_anomalies.py",
            "--json"
        ]
        res = subprocess.run(cmd, cwd=WORKSPACE_ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace")
        self.assertEqual(res.returncode, 0, f"prune anomalies failed: {res.stderr}")
        data = json.loads(res.stdout)
        self.assertIn("anomalies_count", data)
        self.assertIn("anomalies", data)

if __name__ == "__main__":
    unittest.main()
