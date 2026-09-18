#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Core:Audit

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / audit_chain
- **Primary Entrypoints**: `compute_entry_hash`, `AuditChain`, `format_merkle_trail_markdown`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import hashlib
import json
import time
from typing import Dict, Any, List, Optional

GENESIS_HASH = "0000000000000000000000000000000000000000000000000000000000000000"

def compute_entry_hash(prev_hash: str, timestamp: float, agent_id: str, payload: str) -> str:
    """Computes SHA-256 hash chaining entry N from entry N-1"""
    raw_str = f"{prev_hash}|{timestamp:.6f}|{agent_id}|{payload}"
    return hashlib.sha256(raw_str.encode('utf-8')).hexdigest()

class AuditChain:
    """In-memory and persistent hash-chain generator for ALCOA+ compliance"""
    def __init__(self, initial_hash: Optional[str] = None):
        self.last_hash = initial_hash or GENESIS_HASH
        self.entries: List[Dict[str, Any]] = []

    def append(self, agent_id: str, content: str, timestamp: Optional[float] = None) -> Dict[str, Any]:
        ts = timestamp or time.time()
        curr_hash = compute_entry_hash(self.last_hash, ts, agent_id, content)
        entry = {
            "prev_hash": self.last_hash,
            "hash": curr_hash,
            "timestamp": ts,
            "agent_id": agent_id,
            "content": content
        }
        self.entries.append(entry)
        self.last_hash = curr_hash
        return entry

    def verify_chain(self) -> bool:
        """Verifies integrity of the entire chain from genesis to head"""
        expected_prev = GENESIS_HASH
        for entry in self.entries:
            if entry["prev_hash"] != expected_prev:
                return False
            computed = compute_entry_hash(
                expected_prev,
                entry["timestamp"],
                entry["agent_id"],
                entry["content"]
            )
            if computed != entry["hash"]:
                return False
            expected_prev = entry["hash"]
        return True

def format_merkle_trail_markdown(chain_entries: List[Dict[str, Any]]) -> str:
    """Formats chain entries into a clean Markdown Merkle trail for LONG_TERM_MEMORY.md"""
    lines = ["\n### 🛡️ Tamper-Evident Audit Merkle Trail (ALCOA+ Verified)\n"]
    lines.append("| Step | Timestamp | Agent ID | Entry Hash | Prev Hash |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    for idx, e in enumerate(chain_entries, 1):
        prev_short = e['prev_hash'][:8] + "..."
        hash_short = e['hash'][:8] + "..."
        lines.append(f"| {idx} | `{e['timestamp']:.2f}` | `{e['agent_id']}` | `{hash_short}` | `{prev_short}` |")
    return "\n".join(lines)
