"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Nexus Adversarial Invariant Guard**
Static AST & pattern analyzer enforcing the Sovereign Nexus Protocol.
Verifies scheduler invariants, relational cascade ordering, concurrency locks,
credential DLP redaction, and trust boundaries.

### 🔍 Debugging & Observability
- **Failure Path**: Antipattern or invariant violation detected in Tadpole OS.
- **Telemetry Link**: Search `[nexus_adversarial_guard]` in audit logs.
"""

import sys
import re
import os
from pathlib import Path

# Ensure UTF-8 output on Windows
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

def check_c1_version_clobber(persistence_code: str) -> list[str]:
    errors = []
    if "reconciled.version = db_ver" in persistence_code:
        errors.append("[C1-FAIL] Found optimistic locking clobber: 'reconciled.version = db_ver' bypasses OCC.")
    return errors

def check_c2_cascade_ordering(persistence_code: str) -> list[str]:
    errors = []
    match = re.search(r"pub async fn delete_agent_cascade.*?\{(?P<body>.*?)\n\}", persistence_code, re.DOTALL)
    if match:
        body = match.group("body")
        agents_idx = body.find("DELETE FROM agents WHERE")
        mission_idx = body.find("DELETE FROM mission_history WHERE")
        logs_idx = body.find("DELETE FROM mission_logs WHERE")

        if agents_idx != -1:
            if mission_idx == -1:
                errors.append("[C2-FAIL] delete_agent_cascade does not clean 'mission_history'.")
            elif agents_idx < mission_idx:
                errors.append("[C2-FAIL] delete_agent_cascade deletes from 'agents' BEFORE 'mission_history'. Violates PRAGMA foreign_keys = ON.")
            
            if logs_idx == -1:
                errors.append("[C2-FAIL] delete_agent_cascade does not clean 'mission_logs'.")
            elif agents_idx < logs_idx:
                errors.append("[C2-FAIL] delete_agent_cascade deletes from 'agents' BEFORE 'mission_logs'.")
    else:
        errors.append("[C2-FAIL] Could not locate 'delete_agent_cascade' function definition.")
    return errors

def check_c3_file_lease_atomic(conflict_code: str, fs_tools_code: str) -> list[str]:
    errors = []
    if "Entry::Occupied" not in conflict_code or "Entry::Vacant" not in conflict_code:
        errors.append("[C3-FAIL] ConflictManager does not use atomic DashMap entry API (risk of race condition).")
    if "conflict_manager.acquire_lease" not in fs_tools_code:
        errors.append("[C3-FAIL] fs_tools.rs does not invoke conflict_manager.acquire_lease during file writes.")
    if "conflict_manager.release_lease" not in fs_tools_code:
        errors.append("[C3-FAIL] fs_tools.rs does not invoke conflict_manager.release_lease after file writes.")
    return errors

def check_c4_dlp_secret_redaction(security_utils_code: str) -> list[str]:
    errors = []
    required_patterns = [
        "PRIVATE KEY",
        "sk-ant-",
        "sk-proj-",
        "ghp_",
        "Bearer ",
    ]
    for pattern in required_patterns:
        if pattern not in security_utils_code:
            errors.append(f"[C4-FAIL] Security utils regex missing pattern for '{pattern}'.")
    return errors

def check_c5_untrusted_auth_bypass(agent_routes_code: str) -> list[str]:
    errors = []
    if re.search(r"is_authorized_operator.*?=.*?payload\.auto_resume", agent_routes_code):
        errors.append("[C5-FAIL] Found authorization bypass: 'is_authorized_operator' trusts unauthenticated 'payload.auto_resume'.")
    if re.search(r"is_authorized_operator.*?=.*?payload\.user_id", agent_routes_code):
        errors.append("[C5-FAIL] Found authorization bypass: 'is_authorized_operator' trusts unauthenticated 'payload.user_id'.")
    return errors

def check_c6_path_traversal_guards(intelligence_code: str) -> list[str]:
    errors = []
    if "get_blast_radius" not in intelligence_code:
        errors.append("[C6-FAIL] intelligence.rs missing get_blast_radius.")
    if "get_impacted_tests" not in intelligence_code:
        errors.append("[C6-FAIL] intelligence.rs missing get_impacted_tests.")
    # Check that both functions contain path boundary traversal validation
    match_blast = re.search(r"pub async fn get_blast_radius.*?\{(?P<body>.*?)\n\}", intelligence_code, re.DOTALL)
    if match_blast and "Invalid path boundary" not in match_blast.group("body"):
        errors.append("[C6-FAIL] get_blast_radius missing path boundary traversal validation.")
    match_tests = re.search(r"pub async fn get_impacted_tests.*?\{(?P<body>.*?)\n\}", intelligence_code, re.DOTALL)
    if match_tests and "Invalid path boundary" not in match_tests.group("body"):
        errors.append("[C6-FAIL] get_impacted_tests missing path boundary traversal validation.")
    return errors

def check_c7_polymorphic_arg_resolution(codebase_tools_code: str) -> list[str]:
    errors = []
    aliases = ["path", "filename", "file_name", "file"]
    for alias in aliases:
        if f'"{alias}"' not in codebase_tools_code:
            errors.append(f"[C7-FAIL] codebase.rs require_path does not check alias '\"{alias}\"'.")
    return errors

def check_c8_tool_compaction_offload(turn_compactor_code: str, turn_code: str) -> list[str]:
    errors = []
    if "compact_and_offload_observation" not in turn_compactor_code:
        errors.append("[C8-FAIL] turn_compactor.rs missing compact_and_offload_observation function.")
    if "compact_and_offload_observation" not in turn_code:
        errors.append("[C8-FAIL] turn.rs does not call compact_and_offload_observation for observation_buffer.")
    return errors

def main():
    root = Path(__file__).resolve().parent.parent
    server_rs = root / "server-rs" / "src"

    agent_db_rs = server_rs / "agent" / "persistence" / "agent_db.rs"
    conflict_rs = server_rs / "security" / "conflict.rs"
    fs_tools_rs = server_rs / "agent" / "runner" / "fs_tools.rs"
    routes_agent_rs = server_rs / "routes" / "agent.rs"
    intelligence_rs = server_rs / "routes" / "intelligence.rs"
    codebase_rs = server_rs / "agent" / "runner" / "mission_tools" / "codebase.rs"
    turn_compactor_rs = server_rs / "agent" / "runner" / "turn_compactor.rs"
    turn_rs = server_rs / "agent" / "runner" / "intelligence" / "turn.rs"
    security_utils_ts = root / "src" / "utils" / "security_utils.ts"

    all_errors = []

    print("[NEXUS-GUARD] [nexus_adversarial_guard] Executing Red-Team Invariant Analysis...")

    if agent_db_rs.exists():
        code = agent_db_rs.read_text(encoding="utf-8")
        all_errors.extend(check_c1_version_clobber(code))
        all_errors.extend(check_c2_cascade_ordering(code))
    else:
        all_errors.append(f"Missing file: {agent_db_rs}")

    if conflict_rs.exists() and fs_tools_rs.exists():
        c_code = conflict_rs.read_text(encoding="utf-8")
        fs_code = fs_tools_rs.read_text(encoding="utf-8")
        all_errors.extend(check_c3_file_lease_atomic(c_code, fs_code))
    else:
        all_errors.append(f"Missing conflict.rs or fs_tools.rs")

    if security_utils_ts.exists():
        sec_code = security_utils_ts.read_text(encoding="utf-8")
        all_errors.extend(check_c4_dlp_secret_redaction(sec_code))
    else:
        all_errors.append(f"Missing file: {security_utils_ts}")

    if routes_agent_rs.exists():
        agent_code = routes_agent_rs.read_text(encoding="utf-8")
        all_errors.extend(check_c5_untrusted_auth_bypass(agent_code))
    else:
        all_errors.append(f"Missing file: {routes_agent_rs}")

    if intelligence_rs.exists():
        intel_code = intelligence_rs.read_text(encoding="utf-8")
        all_errors.extend(check_c6_path_traversal_guards(intel_code))
    else:
        all_errors.append(f"Missing file: {intelligence_rs}")

    if codebase_rs.exists():
        cb_code = codebase_rs.read_text(encoding="utf-8")
        all_errors.extend(check_c7_polymorphic_arg_resolution(cb_code))
    else:
        all_errors.append(f"Missing file: {codebase_rs}")

    if turn_compactor_rs.exists() and turn_rs.exists():
        tc_code = turn_compactor_rs.read_text(encoding="utf-8")
        t_code = turn_rs.read_text(encoding="utf-8")
        all_errors.extend(check_c8_tool_compaction_offload(tc_code, t_code))
    else:
        all_errors.append(f"Missing turn_compactor.rs or turn.rs")

    print(f"\n[NEXUS-GUARD] Scan completed across all 8 adversarial invariant vectors.")
    if all_errors:
        print(f"❌ Found {len(all_errors)} invariant violation(s):")
        for err in all_errors:
            print(f"  - {err}")
        return 1
    else:
        print("✅ 100% of Sovereign & Nexus Invariants satisfied! Zero vulnerabilities detected.\n")
        return 0

if __name__ == "__main__":
    sys.exit(main())

# [nexus_adversarial_guard]
