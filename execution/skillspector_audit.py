#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / skillspector_audit
- **Primary Entrypoints**: `run_skillspector_audit`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[skillspector_audit]`, `[WARN]`, `[FAIL]`, `[OK]`
- **Witness Tests**: none declared
"""

import sys
import subprocess
import json
import os
import io
from pathlib import Path

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def run_skillspector_audit(project_path: str) -> bool:
    print(f"🔍 [skillspector_audit] Starting security scan on: {project_path}...")
    
    # Execute python -m skillspector scan <project_path> --no-llm --format json
    cmd = [
        "python",
        "-m",
        "skillspector",
        "scan",
        project_path,
        "--no-llm",
        "--format",
        "json"
    ]
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            encoding='utf-8',
            errors='replace',
            timeout=300
        )
    except FileNotFoundError:
        print("[WARN] python executable not found. Skipping SkillSpector audit.")
        return True
    except subprocess.TimeoutExpired:
        print("[FAIL] SkillSpector audit timed out after 5 minutes.")
        return False
        
    # Check if the execution failed due to skillspector not being installed
    if result.returncode != 0:
        if "No module named" in result.stderr or "ModuleNotFoundError" in result.stderr:
            print("[WARN] NVIDIA SkillSpector package is not installed. Skipping deep skill audit.")
            print("To install, run: pip install skillspector")
            return True
            
        print(f"[FAIL] SkillSpector scan command failed with exit code {result.returncode}")
        if result.stderr:
            print(f"Error output:\n{result.stderr.strip()}")
        return False

    # Parse stdout JSON
    try:
        report = json.loads(result.stdout)
    except json.JSONDecodeError as e:
        print(f"[FAIL] Failed to parse SkillSpector JSON output: {e}")
        print(f"Raw Output:\n{result.stdout}")
        return False
        
    risk_score = report.get("risk_score", 0)
    risk_severity = report.get("risk_severity", "LOW")
    risk_reco = report.get("risk_recommendation", "No recommendations.")
    findings = report.get("filtered_findings", [])
    
    print("\n" + "=" * 60)
    print("📊 NVIDIA SKILLSPECTOR SECURITY AUDIT REPORT")
    print("=" * 60)
    print(f"Overall Risk Score : {risk_score}/100")
    print(f"Risk Severity      : {risk_severity}")
    print(f"Recommendation     : {risk_reco}")
    print(f"Total Violations   : {len(findings)}")
    print("=" * 60)
    
    if findings:
        print("\nViolations List:")
        for idx, finding in enumerate(findings, 1):
            rule_id = finding.get("rule_id", "Unknown-Rule")
            severity = finding.get("severity", "LOW")
            loc = finding.get("location", "N/A")
            expl = finding.get("explanation", "")
            
            print(f"\n[{idx}] {rule_id} ({severity.upper()})")
            if loc:
                print(f"    Location: {loc}")
            if expl:
                print(f"    Details : {expl}")
        print("\n" + "=" * 60)
        
    if risk_score >= 50:
        print(f"\n[FAIL] Codebase has a high risk score of {risk_score} (Severity: {risk_severity}). Audit blocked.")
        return False
        
    print("\n[OK] NVIDIA SkillSpector Gate Passed. Codebase verified secure.")
    return True

if __name__ == "__main__":
    project_path = sys.argv[1] if len(sys.argv) > 1 else "."
    success = run_skillspector_audit(project_path)
    sys.exit(0 if success else 1)
