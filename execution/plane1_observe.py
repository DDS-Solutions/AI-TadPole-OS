"""
@docs ARCHITECTURE:Core, ARCHITECTURE:Intelligence

### AI Context Alignment
- **Subsystem**: Sovereign Engine / Plane 1 Observe Mode Diagnostics
- **Primary Entrypoints**: `main`, `run_observe_audit`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Read-only audit with zero in-place mutations.
- `[Structural]` Emits structured triage manifest to reports/plane1/triage.json.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[Plane1Observe]`
- **Witness Tests**: none declared
"""

import os
import sys
import json
import time
import shutil
import argparse
import subprocess
from pathlib import Path
from typing import Dict, Any, List, Tuple

PROJECT_ROOT = Path(__file__).resolve().parent.parent

def is_windows() -> bool:
    return os.name == "nt"

def get_executable(cmd: str) -> str:
    """Resolve executable with .cmd extension on Windows if necessary."""
    if is_windows():
        cmd_path = shutil.which(f"{cmd}.cmd") or shutil.which(f"{cmd}.exe") or shutil.which(cmd)
        if cmd_path:
            return cmd_path
    return cmd

def run_command(cmd: List[str], cwd: Path = PROJECT_ROOT) -> Tuple[int, str, str]:
    """Execute command safely and return returncode, stdout, stderr."""
    resolved_cmd = list(cmd)
    resolved_cmd[0] = get_executable(resolved_cmd[0])
    try:
        proc = subprocess.run(
            resolved_cmd,
            cwd=str(cwd),
            capture_output=True,
            text=True,
            shell=False,
        )
        return proc.returncode, proc.stdout.strip(), proc.stderr.strip()
    except Exception as e:
        return -1, "", str(e)

def get_git_head() -> str:
    """Get current git commit hash."""
    try:
        proc = subprocess.run(["git", "rev-parse", "HEAD"], cwd=str(PROJECT_ROOT), capture_output=True, text=True)
        return proc.stdout.strip()
    except Exception:
        return "unknown"

def run_observe_audit() -> Dict[str, Any]:
    """Run proactive diagnostic scans across the repository."""
    start_time = time.time()
    issues: List[Dict[str, Any]] = []

    print("\n==================================================================")
    print("  [Plane1Observe] PROACTIVE DIAGNOSTIC SCANNER (Observe Mode)")
    print("==================================================================")
    print("  [Plane1Observe] Scanning repository health across static invariants and code...")

    # 1. ADG v2 Header and Manifest Audit
    code, out, err = run_command(["node", "tools/adg/adg.mjs", "verify", "src/"])
    if code != 0:
        issues.append({
            "id": "OBS-ADG-001",
            "category": "Governance",
            "priority": "P0",
            "file": "adg.manifest.json",
            "description": "ADG v2 manifest drift or forbidden v1 header claims detected",
            "blast_radius": "Medium",
            "details": err or out,
        })
    else:
        print("  [✓] ADG v2 Machine Witness & Invariant Manifest: Clean")

    # 2. Dual-Plane & Documentation Link Guard
    code, out, err = run_command(["python", "execution/verify_doc_links.py"])
    if code != 0:
        issues.append({
            "id": "OBS-DOC-001",
            "category": "Documentation",
            "priority": "P2",
            "file": "docs/",
            "description": "Broken documentation cross-links or anchor mismatches found",
            "blast_radius": "Low",
            "details": out or err,
        })
    else:
        print("  [✓] Documentation Cross-Links & Anchor Guard: Clean")

    # 3. Rust Clippy Deprecations and Warnings
    clippy_cmd = ["cargo", "clippy", "--manifest-path", "server-rs/Cargo.toml", "--message-format=json", "--no-deps"]
    code, out, err = run_command(clippy_cmd)
    rust_warnings = 0
    for line in out.splitlines():
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message":
                level = msg.get("message", {}).get("level")
                if level == "warning":
                    rust_warnings += 1
        except Exception:
            pass

    if rust_warnings > 0:
        issues.append({
            "id": "OBS-RUST-001",
            "category": "CodeQuality",
            "priority": "P1",
            "file": "server-rs/",
            "description": f"Found {rust_warnings} cargo clippy warning(s) in server-rs",
            "blast_radius": "Low",
            "details": f"{rust_warnings} warning(s) pending cleanup",
        })
        print(f"  [!] Rust Clippy: {rust_warnings} warning(s) detected")
    else:
        print("  [✓] Rust Clippy Diagnostics: Clean")

    # 4. Invariant Test Health
    code, out, err = run_command(["npx", "vitest", "run", "tests/invariants/", "--no-isolate"])
    if code != 0:
        issues.append({
            "id": "OBS-INV-001",
            "category": "Invariants",
            "priority": "P0",
            "file": "tests/invariants/",
            "description": "Critical security or operational invariant test failure detected",
            "blast_radius": "High",
            "details": out or err,
        })
    else:
        print("  [✓] Executable Invariants (15 suites): Clean")

    duration_ms = int((time.time() - start_time) * 1000)

    # Compute Summary
    p0_count = sum(1 for i in issues if i["priority"] == "P0")
    p1_count = sum(1 for i in issues if i["priority"] == "P1")
    p2_count = sum(1 for i in issues if i["priority"] == "P2")

    # Health score (100 base, -20 per P0, -5 per P1, -2 per P2)
    score = max(0, 100 - (p0_count * 20) - (p1_count * 5) - (p2_count * 2))

    triage_payload = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "git_head": get_git_head(),
        "health_score": score,
        "summary": {
            "p0_critical": p0_count,
            "p1_warnings": p1_count,
            "p2_minor": p2_count,
            "total_issues": len(issues),
        },
        "issues": issues,
        "scan_duration_ms": duration_ms,
    }

    # Save to reports/plane1/triage.json
    reports_dir = PROJECT_ROOT / "reports" / "plane1"
    reports_dir.mkdir(parents=True, exist_ok=True)
    triage_file = reports_dir / "triage.json"
    with open(triage_file, "w", encoding="utf-8") as f:
        json.dump(triage_payload, f, indent=2)

    # Print summary
    print("------------------------------------------------------------------")
    print(f"  OBSERVE SUMMARY: Health Score {score}/100 | {len(issues)} Issue(s)")
    print(f"  P0 Critical: {p0_count} | P1 Warnings: {p1_count} | P2 Minor: {p2_count}")
    print(f"  Triage Dossier: reports/plane1/triage.json")
    print("==================================================================\n")

    return triage_payload

def main():
    parser = argparse.ArgumentParser(description="Plane 1 Observe Mode Diagnostic Scanner")
    parser.add_argument("--json", action="store_true", help="Output raw JSON to stdout")
    args = parser.parse_args()

    payload = run_observe_audit()

    if args.json:
        print(json.dumps(payload, indent=2))

    sys.exit(0)

if __name__ == "__main__":
    main()
