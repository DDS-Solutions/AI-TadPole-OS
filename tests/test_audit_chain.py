#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Core:Audit

### AI Assist Note
**Unit Tests for execution/audit_chain.py (SHA-256 Merkle Audit Chain)**
Verifies ALCOA+ compliance, deterministic hash calculation, chain verification, tamper detection, and Merkle trail formatting.

### 🔍 Debugging & Observability
- **Failure Path**: Hash mismatch or invalid Merkle formatting.
- **Telemetry Link**: Search `[test_audit_chain]` in test execution logs.
"""


import unittest
from execution.audit_chain import AuditChain, compute_entry_hash, format_merkle_trail_markdown, GENESIS_HASH

class TestAuditChain(unittest.TestCase):
    def test_compute_entry_hash_deterministic(self):
        ts = 1700000000.0
        h1 = compute_entry_hash(GENESIS_HASH, ts, "agent_99", "Mission Started")
        h2 = compute_entry_hash(GENESIS_HASH, ts, "agent_99", "Mission Started")
        self.assertEqual(h1, h2)
        self.assertEqual(len(h1), 64)  # Valid SHA-256 hex string

    def test_audit_chain_append_and_verify(self):
        chain = AuditChain()
        entry1 = chain.append("agent_ceo", "Directive issued")
        entry2 = chain.append("agent_coo", "DAG scheduled")
        entry3 = chain.append("agent_99", "Mission completed")

        self.assertEqual(entry1["prev_hash"], GENESIS_HASH)
        self.assertEqual(entry2["prev_hash"], entry1["hash"])
        self.assertEqual(entry3["prev_hash"], entry2["hash"])
        self.assertTrue(chain.verify_chain())

    def test_audit_chain_tamper_detection(self):
        chain = AuditChain()
        chain.append("agent_1", "Log entry 1")
        chain.append("agent_2", "Log entry 2")
        chain.append("agent_3", "Log entry 3")

        self.assertTrue(chain.verify_chain())

        # Tamper with entry 2 payload
        chain.entries[1]["content"] = "Tampered Log entry 2"
        self.assertFalse(chain.verify_chain())

    def test_format_merkle_trail_markdown(self):
        chain = AuditChain()
        chain.append("agent_test", "Test log")
        md = format_merkle_trail_markdown(chain.entries)
        self.assertIn("Tamper-Evident Audit Merkle Trail", md)
        self.assertIn("`agent_test`", md)
        self.assertIn("| Step | Timestamp | Agent ID | Entry Hash | Prev Hash |", md)

if __name__ == "__main__":
    unittest.main()

# Metadata: [test_audit_chain]

