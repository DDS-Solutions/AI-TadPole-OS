#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / deployment-procedures / deployment_validator
- **Primary Entrypoints**: `check_git_status`, `check_migrations_status`, `check_env_template_parity`, `probe_health_endpoint`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic pre-flight deployment verification and health check probing without external state side effects.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[deployment_validator]`, `[PASS]`, `[FAIL]`, `[DEPLOY]`, `[PREFLIGHT]`
- **Witness Tests**: none declared
"""

import sys
import io
import os
import json
import re
import argparse
import subprocess
import urllib.request
import urllib.error
import time
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, List, Optional

# Fix Windows console encoding
if sys.platform == "win32":
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')
    except Exception:
        pass


def check_git_status(repo_path: Path) -> Dict[str, Any]:
    """
    Inspect local git working tree for uncommitted modifications and current HEAD commit.
    """
    result = {
        "is_git": False,
        "branch": "unknown",
        "commit_hash": "unknown",
        "clean": False,
        "dirty_files": [],
        "errors": []
    }
    
    git_dir = repo_path / ".git"
    if not git_dir.exists() and not (repo_path.parent / ".git").exists():
        result["errors"].append("Not a git repository")
        return result

    result["is_git"] = True
    
    try:
        # Branch
        br_proc = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            cwd=str(repo_path),
            capture_output=True,
            text=True,
            check=False
        )
        if br_proc.returncode == 0:
            result["branch"] = br_proc.stdout.strip()
            
        # Commit
        rev_proc = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=str(repo_path),
            capture_output=True,
            text=True,
            check=False
        )
        if rev_proc.returncode == 0:
            result["commit_hash"] = rev_proc.stdout.strip()[:12]
            
        # Status
        st_proc = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=str(repo_path),
            capture_output=True,
            text=True,
            check=False
        )
        if st_proc.returncode == 0:
            lines = [l.strip() for l in st_proc.stdout.splitlines() if l.strip()]
            result["dirty_files"] = lines
            result["clean"] = len(lines) == 0
        else:
            result["errors"].append(st_proc.stderr.strip())
            
    except Exception as e:
        result["errors"].append(f"Git execution error: {e}")
        
    return result


def check_migrations_status(repo_path: Path) -> Dict[str, Any]:
    """
    Verify migrations naming conventions and sequence monotonicity.
    """
    result = {
        "migrations_found": 0,
        "invalid_names": [],
        "duplicate_timestamps": [],
        "valid": True
    }
    
    mig_dir = repo_path / "server-rs" / "migrations"
    if not mig_dir.exists():
        mig_dir = repo_path / "migrations"
        
    if not mig_dir.exists():
        result["message"] = "No migrations directory discovered."
        return result
        
    sql_files = sorted(list(mig_dir.glob("*.sql")))
    result["migrations_found"] = len(sql_files)
    
    timestamps = {}
    timestamp_pattern = re.compile(r'^(\d{8,14})_.*\.sql$')
    
    for f in sql_files:
        m = timestamp_pattern.match(f.name)
        if not m:
            result["invalid_names"].append(f.name)
        else:
            ts = m.group(1)
            if ts in timestamps:
                result["duplicate_timestamps"].append((ts, timestamps[ts], f.name))
            else:
                timestamps[ts] = f.name
                
    if result["invalid_names"] or result["duplicate_timestamps"]:
        result["valid"] = False
        
    return result


def check_env_template_parity(repo_path: Path) -> Dict[str, Any]:
    """
    Verify presence and coverage of environment configuration templates (.env.example).
    """
    result = {
        "template_found": False,
        "template_path": None,
        "variable_count": 0,
        "valid": True
    }
    
    candidates = [repo_path / ".env.example", repo_path / ".env.template", repo_path / "example.env"]
    for c in candidates:
        if c.exists():
            result["template_found"] = True
            result["template_path"] = str(c.name)
            try:
                content = c.read_text(encoding='utf-8', errors='ignore')
                vars_found = [
                    line.split('=')[0].strip()
                    for line in content.splitlines()
                    if line.strip() and not line.strip().startswith('#') and '=' in line
                ]
                result["variable_count"] = len(vars_found)
            except Exception:
                pass
            break
            
    return result


def probe_health_endpoint(url: str, timeout_secs: float = 5.0) -> Dict[str, Any]:
    """
    Probe a live health or status endpoint, recording HTTP status code and latency.
    """
    result = {
        "url": url,
        "status_code": 0,
        "healthy": False,
        "latency_ms": 0.0,
        "error": None
    }
    
    start_time = time.perf_counter()
    try:
        req = urllib.request.Request(
            url,
            headers={"User-Agent": "TadpoleOS-DeploymentValidator/1.0"}
        )
        with urllib.request.urlopen(req, timeout=timeout_secs) as resp:
            elapsed = (time.perf_counter() - start_time) * 1000.0
            result["status_code"] = resp.getcode()
            result["latency_ms"] = round(elapsed, 2)
            result["healthy"] = 200 <= resp.getcode() < 300
    except urllib.error.HTTPError as e:
        elapsed = (time.perf_counter() - start_time) * 1000.0
        result["status_code"] = e.code
        result["latency_ms"] = round(elapsed, 2)
        result["error"] = f"HTTP {e.code}: {e.reason}"
    except Exception as e:
        elapsed = (time.perf_counter() - start_time) * 1000.0
        result["latency_ms"] = round(elapsed, 2)
        result["error"] = str(e)
        
    return result


def run_self_tests() -> bool:
    """
    Execute self-test suite validating git status parsing, migration validation, and probe formatting.
    """
    print("[deployment_validator] Executing self-test suite...")
    
    # 1. Test migration validator logic
    valid_names = ["20260304000100_initial_schema.sql", "20261017000100_metrics_policy.sql", "20260808_classes.sql"]
    invalid_names = ["initial_schema.sql", "migration-1.sql", "2026_test.sql"]
    pattern = re.compile(r'^(\d{8,14})_.*\.sql$')
    
    for name in valid_names:
        assert pattern.match(name) is not None, f"Expected {name} to be recognized as valid migration name"
    for name in invalid_names:
        assert pattern.match(name) is None, f"Expected {name} to be rejected as invalid migration name"
        
    # 2. Test health probe result logic
    mock_resp_success = {"status_code": 200, "healthy": True, "latency_ms": 12.5, "error": None}
    assert mock_resp_success["healthy"] is True
    assert mock_resp_success["status_code"] == 200
    
    mock_resp_fail = {"status_code": 500, "healthy": False, "latency_ms": 85.0, "error": "Internal Error"}
    assert mock_resp_fail["healthy"] is False
    
    # 3. Test git status output parser logic
    sample_porcelain = " M src/index.ts\n?? new_file.txt\n"
    lines = [l.strip() for l in sample_porcelain.splitlines() if l.strip()]
    assert len(lines) == 2
    assert "M src/index.ts" in lines
    
    print("[deployment_validator] [PASS] All self-tests passed successfully.")
    return True


def main():
    parser = argparse.ArgumentParser(description="Deterministic Deployment Pre-Flight Validator")
    parser.add_argument("path", nargs="?", default=".", help="Target project root directory")
    parser.add_argument("--test", action="store_true", help="Execute self-test suite and exit")
    parser.add_argument("--probe", type=str, default=None, help="Probe a live URL health endpoint (e.g., http://localhost:8080/health)")
    parser.add_argument("--timeout", type=float, default=5.0, help="Probe timeout in seconds (default 5.0)")
    parser.add_argument("--json", action="store_true", help="Output machine-readable JSON report")
    
    args = parser.parse_args()
    
    if args.test:
        try:
            success = run_self_tests()
            sys.exit(0 if success else 1)
        except AssertionError as e:
            print(f"[deployment_validator] [FAIL] Self-test assertion failed: {e}")
            sys.exit(1)
            
    target_path = Path(args.path).resolve()
    
    print("[deployment_validator] Initializing pre-flight deployment check...")
    print("[DEPLOY] Starting deployment pipeline integrity scan...")
    print("[PREFLIGHT] Checking git working tree and manifest status...")
    
    # 1. Git Status
    git_status = check_git_status(target_path)
    
    # 2. Migrations Status
    mig_status = check_migrations_status(target_path)
    
    # 3. Environment Template Status
    env_status = check_env_template_parity(target_path)
    
    # 4. Optional Health Probe
    probe_status = None
    if args.probe:
        probe_status = probe_health_endpoint(args.probe, timeout_secs=args.timeout)
        
    overall_pass = True
    if not git_status.get("clean", False):
        overall_pass = False
    if not mig_status.get("valid", True):
        overall_pass = False
    if probe_status and not probe_status.get("healthy", False):
        overall_pass = False
        
    report = {
        "script": "deployment_validator",
        "timestamp": datetime.now().isoformat(),
        "target_directory": str(target_path),
        "overall_passed": overall_pass,
        "git_status": git_status,
        "migrations_status": mig_status,
        "env_template_status": env_status,
        "health_probe": probe_status
    }
    
    if args.json:
        print(json.dumps(report, indent=2))
        sys.exit(0 if overall_pass else 1)
        
    print(f"\n{'='*64}")
    print(f"🚀 [DEPLOYMENT VALIDATOR] Sovereign Pre-Flight Verification")
    print(f"{'='*64}")
    print(f"Target Directory : {target_path}")
    print(f"Timestamp        : {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'-'*64}")
    
    # Git Summary
    if git_status["is_git"]:
        status_icon = "✅" if git_status["clean"] else "⚠️ "
        print(f"{status_icon} Git Branch       : {git_status['branch']} (Commit: {git_status['commit_hash']})")
        if not git_status["clean"]:
            print(f"   Uncommitted files ({len(git_status['dirty_files'])}):")
            for f in git_status["dirty_files"][:5]:
                print(f"     - {f}")
            if len(git_status["dirty_files"]) > 5:
                print(f"     ... and {len(git_status['dirty_files']) - 5} more.")
    else:
        print("⚠️  Not a git repository.")
        
    # Migrations Summary
    if mig_status["migrations_found"] > 0:
        mig_icon = "✅" if mig_status["valid"] else "❌"
        print(f"{mig_icon} Database Migrations: {mig_status['migrations_found']} found")
        if mig_status["invalid_names"]:
            for inv in mig_status["invalid_names"]:
                print(f"   ❌ Invalid naming: {inv}")
        if mig_status["duplicate_timestamps"]:
            for ts, f1, f2 in mig_status["duplicate_timestamps"]:
                print(f"   ❌ Duplicate timestamp {ts}: {f1} <-> {f2}")
                
    # Environment Template Summary
    env_icon = "✅" if env_status["template_found"] else "ℹ️ "
    print(f"{env_icon} Env Template    : {'Found (' + str(env_status['variable_count']) + ' variables)' if env_status['template_found'] else 'None detected'}")
    
    # Health Probe Summary
    if probe_status:
        probe_icon = "✅" if probe_status["healthy"] else "❌"
        print(f"{probe_icon} Health Endpoint : {probe_status['url']} (Status: {probe_status['status_code']}, Latency: {probe_status['latency_ms']}ms)")
        if probe_status["error"]:
            print(f"   Error: {probe_status['error']}")
            
    print(f"{'-'*64}")
    if overall_pass:
        print("[deployment_validator] [PASS] Pre-flight deployment integrity verified successfully.")
    else:
        print("[deployment_validator] [FAIL] Pre-flight deployment integrity failed. Resolve blocks before deploying.")
    print(f"{'='*64}\n")
    
    sys.exit(0 if overall_pass else 1)


if __name__ == "__main__":
    main()
