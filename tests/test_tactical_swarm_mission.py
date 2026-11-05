"""
@docs ARCHITECTURE:Core:Audit

### AI Context Alignment
- **Subsystem**: Test Verification Suite / test_tactical_swarm_mission
- **Primary Entrypoints**: `TestTacticalSwarmMission`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic internal state integrity and strict interface contract compliance.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import unittest
from pathlib import Path
from execution.run_tactical_swarm_mission import (
    TACTICAL_DIRECTIVE,
    TARGET_AGENTS,
    TacticalSwarmRunner
)


class TestTacticalSwarmMission(unittest.TestCase):

    def test_mission_specification_artifacts(self):
        doc_path = Path("docs/tactical_swarm_mission.md")
        self.assertTrue(doc_path.exists(), "tactical_swarm_mission.md specification file must exist")
        content = doc_path.read_text(encoding="utf-8")
        self.assertIn("The Tactical Swarm", content)
        self.assertIn("agent_db.rs", content)
        self.assertIn("RunContext.lineage", content)

    def test_directive_contract(self):
        self.assertIn("Sovereign Swarm", TACTICAL_DIRECTIVE)
        self.assertIn("agent_db.rs", TACTICAL_DIRECTIVE)
        self.assertEqual(TARGET_AGENTS, ["1", "2", "alpha", "coder"])

    def test_report_generation(self):
        runner = TacticalSwarmRunner()
        before = {
            "1": {"tokens_used": 1000, "cost_usd": 0.01, "name": "CEO", "role": "Strategic Lead"},
            "2": {"tokens_used": 2000, "cost_usd": 0.02, "name": "COO", "role": "Operations Lead"},
            "alpha": {"tokens_used": 500, "cost_usd": 0.005, "name": "Alpha", "role": "Commander"},
            "coder": {"tokens_used": 800, "cost_usd": 0.008, "name": "Coder", "role": "Developer"}
        }
        after = {
            "1": {"tokens_used": 1500, "cost_usd": 0.015, "name": "CEO", "role": "Strategic Lead", "status": "completed"},
            "2": {"tokens_used": 2500, "cost_usd": 0.025, "name": "COO", "role": "Operations Lead", "status": "idle"},
            "alpha": {"tokens_used": 1000, "cost_usd": 0.010, "name": "Alpha", "role": "Commander", "status": "idle"},
            "coder": {"tokens_used": 1600, "cost_usd": 0.016, "name": "Coder", "role": "Developer", "status": "idle"}
        }
        
        runner.generate_report("test-task-123", before, after)
        report_file = Path(".tmp/tactical_swarm_report.md")
        self.assertTrue(report_file.exists())
        report_content = report_file.read_text(encoding="utf-8")
        self.assertIn("Tactical Swarm Execution Audit Report", report_content)
        self.assertIn("+500", report_content) # CEO delta
        self.assertIn("+800", report_content) # Coder delta


if __name__ == "__main__":
    unittest.main()
