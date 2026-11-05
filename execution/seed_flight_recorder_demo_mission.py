#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / seed_flight_recorder_demo_mission
- **Primary Entrypoints**: `compute_log_hash`, `run_mission_tail`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic generation of flight recorder demonstration mission.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""
"""
Sovereign Flight Recorder Demonstration Mission Generator & Tailing Pipeline.

This script executes a comprehensive sovereign mission that populates all 5 KPI cards
and sub-tabs in the Tadpole OS Mission Flight Recorder & Run History dashboard:
1. Status & Merkle Integrity (ALCOA+ cryptographic chain proof)
2. Micro-Credit Economics & 2PC Transaction Ledger ($0.042 / $0.100 budget burn)
3. Multi-Turn Merkle Audit Trail (Agent, User, System, with Info, Warning, Error severities)
4. CAS Mutation Ledger (Live line-by-line diffs for workspace source files)
5. Agent 99 Behavioral Drift Debrief (LanceDB vector space drift score 0.0240, recommendations)
6. Swarm Memory Palace (Categorized topic rooms and findings)
7. TrustGraph Entity & Relationship Edges
8. Long-Term Retention Pin (Reaper Pin active)

Tails execution for >= 34 seconds to demonstrate live incremental flight progress.
"""

import sys
import os
import time
import json
import sqlite3
import hashlib
import uuid
from datetime import datetime, timezone

# Ensure UTF-8 output on Windows consoles
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

MISSION_ID = "mission-sovereign-flight-99"
AGENT_ID = "agent-sovereign-alpha"
TITLE = "Sovereign Swarm Autonomous Governance & CAS Consensus Verification"
DB_PATH = os.path.join("data", "state", "tadpole.db")

def compute_log_hash(prev_hash, mission_id, agent_id, source, text, timestamp_iso):
    """
    Computes canonical SHA-256 Merkle chain hash matching server-rs:
    SHA256(prev_hash + mission_id + agent_id + source + text + timestamp_iso)
    """
    hasher = hashlib.sha256()
    if prev_hash:
        hasher.update(prev_hash.encode("utf-8"))
    hasher.update(mission_id.encode("utf-8"))
    hasher.update(agent_id.encode("utf-8"))
    hasher.update(source.encode("utf-8"))
    hasher.update(text.encode("utf-8"))
    hasher.update(timestamp_iso.encode("utf-8"))
    return hasher.hexdigest()

def get_db():
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute("PRAGMA busy_timeout=5000;")
    return conn

def cleanup_previous_run(conn):
    print(f"[*] Purging previous artifacts for {MISSION_ID}...")
    cur = conn.cursor()
    cur.execute("DELETE FROM mission_logs WHERE mission_id = ?", (MISSION_ID,))
    cur.execute("DELETE FROM file_revisions WHERE mission_id = ?", (MISSION_ID,))
    cur.execute("DELETE FROM swarm_context WHERE mission_id = ?", (MISSION_ID,))
    cur.execute("DELETE FROM graph_entities WHERE mission_id = ?", (MISSION_ID,))
    cur.execute("DELETE FROM graph_relations WHERE mission_id = ?", (MISSION_ID,))
    cur.execute("DELETE FROM transaction_ledger WHERE buyer_address = ? OR seller_address = ?", (MISSION_ID, MISSION_ID))
    cur.execute("DELETE FROM transaction_ledger WHERE buyer_address = ? OR seller_address = ?", (AGENT_ID, AGENT_ID))
    cur.execute("DELETE FROM mission_history WHERE id = ?", (MISSION_ID,))
    conn.commit()

def seed_cas_and_diffs(conn):
    print("[*] Seeding CAS Blobs & File Revisions for live diff rendering...")
    cur = conn.cursor()
    now_iso = datetime.now(timezone.utc).isoformat()

    # Target File 1: server-rs/src/main.rs
    main_path = os.path.join("server-rs", "src", "main.rs")
    if os.path.exists(main_path):
        with open(main_path, "r", encoding="utf-8") as f:
            actual_content = f.read()
    else:
        actual_content = "fn main() {\n    println!(\"Tadpole Engine\");\n}\n"

    # Create distinct pre-mutation content for diff demonstration
    pre_mutation_main = (
        "// [PRE-FLIGHT CAS SNAPSHOT] Legacy unverified runtime entrypoint\n"
        "// Pending Sovereign Swarm architectural consensus\n\n"
        + "\n".join(actual_content.splitlines()[:40])
        + "\n// [END PRE-MUTATION PREVIEW]\n"
    )
    blob_hash_main = hashlib.sha256(pre_mutation_main.encode("utf-8")).hexdigest()

    cur.execute(
        "INSERT OR REPLACE INTO cas_blobs (hash, content, size_bytes, created_at) VALUES (?, ?, ?, ?)",
        (blob_hash_main, pre_mutation_main.encode("utf-8"), len(pre_mutation_main), now_iso)
    )
    cur.execute(
        """INSERT INTO file_revisions 
        (workspace_id, file_path, hash, content, size_bytes, version_num, mission_id, agent_id, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        ("default", "server-rs/src/main.rs", blob_hash_main, pre_mutation_main.encode("utf-8"), 
         len(pre_mutation_main), 1, MISSION_ID, AGENT_ID, now_iso)
    )

    # Target File 2: src/App.tsx
    app_path = os.path.join("src", "App.tsx")
    if os.path.exists(app_path):
        with open(app_path, "r", encoding="utf-8") as f:
            actual_app = f.read()
    else:
        actual_app = "export default function App() { return <div>App</div>; }\n"

    pre_mutation_app = (
        "/**\n * @deprecated Legacy App Shell Snapshot\n * Prior to Mission Flight Recorder UI/UX Enhancement\n */\n"
        + "\n".join(actual_app.splitlines()[:50])
        + "\n// End legacy app snapshot\n"
    )
    blob_hash_app = hashlib.sha256(pre_mutation_app.encode("utf-8")).hexdigest()

    cur.execute(
        "INSERT OR REPLACE INTO cas_blobs (hash, content, size_bytes, created_at) VALUES (?, ?, ?, ?)",
        (blob_hash_app, pre_mutation_app.encode("utf-8"), len(pre_mutation_app), now_iso)
    )
    cur.execute(
        """INSERT INTO file_revisions 
        (workspace_id, file_path, hash, content, size_bytes, version_num, mission_id, agent_id, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        ("default", "src/App.tsx", blob_hash_app, pre_mutation_app.encode("utf-8"), 
         len(pre_mutation_app), 1, MISSION_ID, AGENT_ID, now_iso)
    )
    conn.commit()
    print("    -> Committed 2 CAS revisions (server-rs/src/main.rs, src/App.tsx)")

def seed_memory_palace(conn):
    print("[*] Seeding Swarm Memory Palace findings across topic rooms...")
    cur = conn.cursor()
    now_iso = datetime.now(timezone.utc).isoformat()

    findings = [
        ("architectural_invariants", AGENT_ID, 
         "Active Documentation Guard (ADG v2) verified: 0 hand-written witness assertions detected in headers. AST symbol blast radius scoped to 57 nodes."),
        ("economic_settlement", AGENT_ID,
         "Two-Phase Commit (2PC) micro-credit escrow guarantees atomic idempotency across parallel subagent worker tasks. Zero double-spend verified."),
        ("zero_trust_security", "agent-sec-auditor",
         "Sovereign capability keys rotated dynamically every 3,600s with SHA-256 HMAC cryptographic challenge response. Replay attacks neutralized."),
        ("architectural_invariants", "99",
         "Zustand state store slice boundaries and IPC bridge commands verified against open telemetry trace spans with zero leaking state.")
    ]

    for topic, agent, finding in findings:
        cur.execute(
            """INSERT INTO swarm_context (id, mission_id, agent_id, topic, finding, timestamp)
            VALUES (?, ?, ?, ?, ?, ?)""",
            (str(uuid.uuid4()), MISSION_ID, agent, topic, finding, now_iso)
        )
    conn.commit()
    print(f"    -> Seeded {len(findings)} Memory Palace findings in 3 topic rooms.")

def seed_trust_graph(conn):
    print("[*] Seeding TrustGraph entities and relationship edges...")
    cur = conn.cursor()
    now_ts = int(time.time())

    entities = [
        ("ent-agent-alpha", "Agent Sovereign Alpha", "AGENT", "Primary autonomous mission orchestrator and coordinator", MISSION_ID, now_ts),
        ("ent-agent-99", "Agent 99 QA Auditor", "AGENT", "Principal verification auditor and semantic drift evaluator", MISSION_ID, now_ts),
        ("ent-cas-storage", "Content Addressable Storage (CAS)", "SERVICE", "Deduplicated cryptographic blob mutation ledger", MISSION_ID, now_ts),
        ("ent-trust-chain", "ALCOA+ Merkle Chain", "CONCEPT", "Continuous SHA-256 tamper-evident flight recording substrate", MISSION_ID, now_ts),
    ]

    for eid, name, ntype, desc, mid, ts in entities:
        cur.execute(
            """INSERT OR REPLACE INTO graph_entities (id, name, type, description, mission_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?)""",
            (eid, name, ntype, desc, mid, ts)
        )

    relations = [
        ("rel-01", "ent-agent-alpha", "ent-cas-storage", "MUTATES", 0.95, MISSION_ID, now_ts),
        ("rel-02", "ent-agent-99", "ent-trust-chain", "AUDITED_BY", 1.00, MISSION_ID, now_ts),
        ("rel-03", "ent-agent-alpha", "ent-trust-chain", "DEPENDS_ON", 0.90, MISSION_ID, now_ts),
        ("rel-04", "ent-cas-storage", "ent-trust-chain", "PROVES", 0.98, MISSION_ID, now_ts),
    ]

    for rid, src, tgt, rtype, weight, mid, ts in relations:
        cur.execute(
            """INSERT OR REPLACE INTO graph_relations (id, source_entity_id, target_entity_id, relation_type, weight, mission_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (rid, src, tgt, rtype, weight, mid, ts)
        )

    conn.commit()
    print(f"    -> Seeded {len(entities)} TrustGraph entities and {len(relations)} relational edges.")

def seed_transaction_ledger(conn):
    print("[*] Seeding 2PC Economic Transaction Ledger...")
    cur = conn.cursor()
    now_ts = int(time.time())

    txs = [
        ("tx-2pc-001", "ESCROW_RESERVE", AGENT_ID, "treasury-escrow", "compute-quantum-hours", 24, "SETTLED", "sig-sha256-4b9e28f01a7c", now_ts),
        ("tx-2pc-002", "LEDGER_TRANSFER", AGENT_ID, "lancedb-vector-vault", "dense-vector-tier1", 18, "SETTLED", "sig-sha256-8a1c92d34e5f", now_ts + 2),
    ]

    for tid, ttype, buyer, seller, asset, amount, status, sig, ts in txs:
        cur.execute(
            """INSERT OR REPLACE INTO transaction_ledger 
            (id, transaction_type, buyer_address, seller_address, asset_id, amount, status, audit_signature, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (tid, ttype, buyer, seller, asset, amount, status, sig, ts)
        )

    conn.commit()
    print(f"    -> Seeded {len(txs)} transactions totaling 42 credits ($0.042 USD).")

def run_mission_tail():
    conn = get_db()
    cleanup_previous_run(conn)

    # Initialize Header in ACTIVE state
    print(f"[*] Initializing mission header '{TITLE}' [STATUS=ACTIVE]...")
    cur = conn.cursor()
    cur.execute(
        """INSERT INTO mission_history (id, agent_id, title, status, budget_usd, cost_usd, is_pinned, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)""",
        (MISSION_ID, AGENT_ID, TITLE, "active", 0.100, 0.042, 1)
    )
    conn.commit()

    seed_cas_and_diffs(conn)
    seed_memory_palace(conn)
    seed_trust_graph(conn)
    seed_transaction_ledger(conn)

    # Multi-turn execution steps tailored to run over >= 34 seconds
    steps = [
        {
            "delay": 0,
            "source": "system",
            "agent_id": AGENT_ID,
            "severity": "info",
            "text": "[Genesis Flight Stage] Initializing Sovereign Swarm runtime session. Merkle audit trail established at genesis block.",
            "metadata": {"stage": "genesis", "protocol_version": "v2.4", "target_env": "production-hardened"}
        },
        {
            "delay": 5,
            "source": "user",
            "agent_id": "entity-0-overlord",
            "severity": "info",
            "text": "Deploy sovereign swarm to audit CAS mutation pipeline, verify 2PC token economic escrow, and register TrustGraph relationships.",
            "metadata": {"origin": "UI-Oversight-Console", "clearance": "ENTITY_0_MAX"}
        },
        {
            "delay": 5,
            "source": "agent",
            "agent_id": AGENT_ID,
            "severity": "info",
            "text": "Scanning workspace symbol AST. Ingested 57 node boundaries and detected active Tauri IPC bridge contracts.",
            "metadata": {"blast_radius_nodes": 57, "tool": "graph:blast:guard", "duration_ms": 112}
        },
        {
            "delay": 6,
            "source": "system",
            "agent_id": AGENT_ID,
            "severity": "warning",
            "text": "Transient memory pressure warning: Token buffer reached 82% threshold during LanceDB dense vector embedding. Cache eviction executed successfully.",
            "metadata": {"warning_code": "WARN_MEM_PRESSURE", "buffer_usage_pct": 82.4, "action_taken": "compacted_cache"}
        },
        {
            "delay": 6,
            "source": "agent",
            "agent_id": AGENT_ID,
            "severity": "info",
            "text": "CAS File Mutation executed. Committed pre-mutation snapshot to cas_blobs and updated file_revisions for server-rs and UI components.",
            "metadata": {"mutated_files": 2, "cas_algorithm": "SHA256_DEDUP", "atomic_tx": "tx-2pc-001"}
        },
        {
            "delay": 6,
            "source": "agent",
            "agent_id": AGENT_ID,
            "severity": "info",
            "text": "Atomic 2PC micro-credit escrow settled. 42 credits transferred from buyer agent-sovereign-alpha to seller treasury-escrow.",
            "metadata": {"escrow_id": "escrow-9981-sec", "status": "SETTLED", "signature": "0x4a7e9b21f8c0d9"}
        },
        {
            "delay": 6,
            "source": "agent",
            "agent_id": "99",
            "severity": "info",
            "text": "Agent 99 QA Auditor Final Debrief: All ALCOA+ Merkle chain links verified without defect. Behavioral drift in LanceDB vector space is 0.0240 (nominal < 0.1500). System state transitions verified and certified for production.",
            "metadata": {
                "drift_score": 0.024,
                "auditor": "Agent 99",
                "verdict": "VERIFIED_SUCCESS",
                "recommendations": [
                    "Enforce strict schema validation on CAS commits",
                    "Index high-frequency relations in TrustGraph",
                    "Rotate 2PC economic escrow keys every 3600s"
                ]
            }
        }
    ]

    print("\n" + "=" * 70)
    print("  🚀 COMMENCING SOVEREIGN FLIGHT EXECUTION (TAILING >= 34 SECONDS)")
    print("=" * 70)

    start_time = time.time()
    prev_hash = None

    for idx, step in enumerate(steps):
        delay = step["delay"]
        if delay > 0:
            print(f"⏳ Tailing progress... waiting {delay}s for next turn [Elapsed: {time.time() - start_time:.1f}s]")
            time.sleep(delay)

        log_id = str(uuid.uuid4())
        # Use exact ISO8601 formatting with timezone offset to match chrono DateTime<Utc>
        now_dt = datetime.now(timezone.utc)
        timestamp_iso = now_dt.strftime("%Y-%m-%dT%H:%M:%S.%f") + "+00:00"

        # Authoritative Merkle Hash computation
        curr_hash = compute_log_hash(
            prev_hash=prev_hash,
            mission_id=MISSION_ID,
            agent_id=step["agent_id"],
            source=step["source"],
            text=step["text"],
            timestamp_iso=timestamp_iso
        )

        meta_json = json.dumps(step["metadata"])

        cur.execute(
            """INSERT INTO mission_logs 
            (id, mission_id, agent_id, source, text, severity, timestamp, metadata, hash, prev_hash)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (log_id, MISSION_ID, step["agent_id"], step["source"], step["text"], 
             step["severity"], timestamp_iso, meta_json, curr_hash, prev_hash)
        )
        conn.commit()

        print(f"  [Turn {idx}] {step['source'].upper():<6} | {step['severity'].upper():<7} | Hash: {curr_hash[:12]}... | Prev: {(prev_hash[:12] + '...') if prev_hash else 'GENESIS'}")
        print(f"           Text: {step['text'][:80]}...")
        prev_hash = curr_hash

    # Finalize mission state to COMPLETED
    print("\n[*] Concluding mission flight stage. Updating status -> COMPLETED...")
    cur.execute(
        "UPDATE mission_history SET status = 'completed', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        (MISSION_ID,)
    )
    conn.commit()
    conn.close()

    total_elapsed = time.time() - start_time
    print(f"\n✅ Mission {MISSION_ID} execution concluded in {total_elapsed:.2f}s (target: >= 30s).")
    print("=" * 70 + "\n")

if __name__ == "__main__":
    run_mission_tail()
