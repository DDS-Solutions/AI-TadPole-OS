#!/usr/bin/env python3
import subprocess
import os
import sys
import json
import io
from datetime import datetime
from pathlib import Path

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

# --- Configuration ---
ROOT = Path(__file__).resolve().parent.parent
REPORTS_DIR = ROOT / "reports"
TIMESTAMP = datetime.now().strftime("%Y%m%d_%H%M%S")
REPORT_FILE = REPORTS_DIR / f"SOVEREIGN_AUDIT_{TIMESTAMP}.md"

AUDIT_SUITE = [
    {
        "name": "Security Scan (P0)",
        "cmd": ["python", ".agent/skills/vulnerability-scanner/scripts/security_scan.py", "."],
        "critical": True
    },
    {
        "name": "Documentation Parity (P1)",
        "cmd": ["python", "execution/parity_guard.py", "."],
        "critical": True
    },
    {
        "name": "Sovereign Awakening (P1)",
        "cmd": ["python", "execution/awaken.py", "--check"],
        "critical": True
    },
    {
        "name": "API Reference Consistency",
        "cmd": ["python", "execution/generate_api_reference.py"],
        "critical": False
    }
]

def run_audit_command(name, cmd):
    print(f"🔄 Running {name}...")
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT)
        passed = result.returncode == 0
        return {
            "name": name,
            "passed": passed,
            "stdout": result.stdout,
            "stderr": result.stderr
        }
    except Exception as e:
        return {
            "name": name,
            "passed": False,
            "stdout": "",
            "stderr": str(e)
        }

def main():
    if not REPORTS_DIR.exists():
        REPORTS_DIR.mkdir(parents=True)

    print(f"--- 🛡️ Tadpole OS Master Sovereign Audit (v1.1.5) ---")
    results = []
    
    for audit in AUDIT_SUITE:
        res = run_audit_command(audit["name"], audit["cmd"])
        results.append(res)
        if not res["passed"]:
            print(f"  ❌ {audit['name']} FAILED")
            if audit["critical"]:
                print(f"  [!] CRITICAL FAILURE. Stopping audit.")
                # We continue to generate report though
        else:
            print(f"  ✅ {audit['name']} PASSED")

    # Generate Markdown Report
    passed_all = all(r["passed"] for r in results if next(a for a in AUDIT_SUITE if a["name"] == r["name"])["critical"])
    
    report = [
        f"# 🛡️ Sovereign Audit Report",
        f"**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
        f"**Status**: {'🟢 PASS' if passed_all else '🔴 FAIL'}",
        f"**Engine Version**: 1.1.5",
        "",
        "## 📊 Executive Summary",
        "| Audit Pillar | Status | Notes |",
        "| :--- | :--- | :--- |"
    ]

    for r in results:
        status = "✅ PASS" if r["passed"] else "❌ FAIL"
        critical_tag = " (CRITICAL)" if next(a for a in AUDIT_SUITE if a["name"] == r["name"])["critical"] else ""
        report.append(f"| {r['name']}{critical_tag} | {status} | {'See details below' if not r['passed'] else 'Compliant'} |")

    report.append("\n## 🔍 Detailed Findings\n")
    
    for r in results:
        report.append(f"### {r['name']}")
        if r["passed"]:
            report.append("Status: `COMPLIANT`")
        else:
            report.append("Status: `DRIFT DETECTED`")
            report.append(f"#### Stdout\n```\n{r['stdout'][-2000:]}\n```")
            if r["stderr"]:
                report.append(f"#### Stderr\n```\n{r['stderr'][-2000:]}\n```")
        report.append("\n---")

    with open(REPORT_FILE, 'w', encoding='utf-8') as f:
        f.write("\n".join(report))

    print(f"\nFinal Report Generated: {REPORT_FILE}")
    
    if not passed_all:
        sys.exit(1)
    sys.exit(0)

if __name__ == "__main__":
    main()
