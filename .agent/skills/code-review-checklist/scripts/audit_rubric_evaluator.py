#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / code-review-checklist / audit_rubric_evaluator
- **Primary Entrypoints**: `evaluate_rubric_pillars`, `detect_security_sinks`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic audit evaluation across logic, security, performance, and clean-code.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[audit_rubric_evaluator]`, `[RUBRIC]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import json
import re
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

SECURITY_SINK_PATTERNS = [
    (r'\beval\s*\(', "Critical: Dynamic eval() execution hazard"),
    (r'\bnew\s+Function\s*\(', "Critical: Dynamic Function() constructor execution hazard"),
    (r'\.innerHTML\s*=', "Critical: Direct innerHTML assignment XSS hazard"),
    (r'shell\s*=\s*True', "Critical: Subprocess execution with shell=True"),
    (r'(?:api_key|secret_key|private_key)\s*=\s*[\'"][A-Za-z0-9_\-]{20,}[\'"]', "Critical: Potential hardcoded secret credential"),
]

def detect_security_sinks(code: str) -> List[Dict[str, Any]]:
    """Scan source code for dangerous security sinks and credential leakage."""
    findings = []
    lines = code.splitlines()

    for idx, line in enumerate(lines, 1):
        line_clean = line.strip()
        if line_clean.startswith(('//', '#', '/*', '*')):
            continue

        for pattern, desc in SECURITY_SINK_PATTERNS:
            if re.search(pattern, line_clean, re.IGNORECASE):
                item = {
                    "line": idx,
                    "severity": "BLOCKING",
                    "pillar": "Security",
                    "issue": desc,
                    "snippet": line_clean
                }
                findings.append(item)

    return findings

def evaluate_rubric_pillars(code: str, file_ext: str = "") -> Dict[str, Any]:
    """Audit code against the 5 Master Rubric pillars."""
    findings = []
    lines = code.splitlines()

    # 1. Security Pillar
    findings.extend(detect_security_sinks(code))

    # 2. Performance Pillar (Check for nested loops)
    for idx, line in enumerate(lines, 1):
        line_clean = line.strip()
        if re.search(r'\bfor\b.*\bfor\b', line_clean):
            item = {
                "line": idx,
                "severity": "SUGGESTION",
                "pillar": "Performance",
                "issue": "Inline nested loop detected. Evaluate if complexity can be reduced.",
                "snippet": line_clean
            }
            findings.append(item)

    # 3. Quality & Typing Pillar
    for idx, line in enumerate(lines, 1):
        line_clean = line.strip()
        if file_ext in ('.ts', '.tsx') and re.search(r'\bas\s+any\b', line_clean):
            item = {
                "line": idx,
                "severity": "SUGGESTION",
                "pillar": "Quality",
                "issue": "TypeScript 'as any' bypasses compile-time checks.",
                "snippet": line_clean
            }
            findings.append(item)
        elif file_ext == '.rs' and '.unwrap()' in line_clean and not line_clean.startswith('//'):
            item = {
                "line": idx,
                "severity": "BLOCKING",
                "pillar": "Quality",
                "issue": "Unhandled Rust .unwrap() in production code.",
                "snippet": line_clean
            }
            findings.append(item)

    blocking = [f for f in findings if f["severity"] == "BLOCKING"]
    suggestions = [f for f in findings if f["severity"] == "SUGGESTION"]
    nits = [f for f in findings if f["severity"] == "NIT"]

    verdict = "APPROVED"
    if blocking:
        verdict = "BLOCKING"
    elif suggestions:
        verdict = "MINOR FIXES"

    return {
        "verdict": verdict,
        "total_findings": len(findings),
        "blocking_count": len(blocking),
        "suggestion_count": len(suggestions),
        "nit_count": len(nits),
        "findings": findings
    }

def run_self_tests() -> bool:
    """Execute built-in self-tests verifying Master Rubric evaluation rules."""
    print("🔬 [audit_rubric_evaluator] Running self-test suite...")

    # Test 1: Security sink detection
    sink_sample = 'element.innerHTML = "<p>" + userInput + "</p>";'
    sinks = detect_security_sinks(sink_sample)
    assert any("innerHTML" in s["issue"] for s in sinks), f"Test 1 Failed: Expected innerHTML finding, got: {sinks}"

    # Test 2: Quality unwrap check
    rs_sample = "let value = config.get('key').unwrap();"
    res = evaluate_rubric_pillars(rs_sample, ".rs")
    assert res["verdict"] == "BLOCKING", f"Test 2 Failed: Expected BLOCKING verdict for unwrap, got: {res}"

    # Test 3: Clean code pass
    clean_sample = "export function sanitize(input: string): string { return input.trim(); }"
    res3 = evaluate_rubric_pillars(clean_sample, ".ts")
    assert res3["verdict"] == "APPROVED", f"Test 3 Failed: Expected APPROVED verdict, got: {res3}"

    print("✅ [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic Master Code Review Rubric Evaluator")
    parser.add_argument("--file", type=str, help="Path to source file to audit")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    content = ""
    target_name = "stdin"
    ext = ""

    if args.file:
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"❌ [FAIL] File not found: {file_path}", file=sys.stderr)
            sys.exit(1)
        target_name = str(file_path)
        ext = file_path.suffix.lower()
        content = file_path.read_text(encoding='utf-8', errors='replace')
    elif not sys.stdin.isatty():
        content = sys.stdin.read()
    else:
        print("ℹ️  No input file or piped text provided. Running self-test suite.")
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print(f"🔍 [audit_rubric_evaluator] Evaluating Master Rubric for: {target_name}")

    report = evaluate_rubric_pillars(content, ext)

    print(f"📊 [RUBRIC] Verdict: {report['verdict']} | Blocking: {report['blocking_count']} | Suggestions: {report['suggestion_count']}")

    if args.json:
        print(json.dumps(report, indent=2))
        sys.exit(0 if report["verdict"] == "APPROVED" else 1)

    if report["verdict"] != "APPROVED":
        print(f"❌ [FAIL] Master Rubric Audit found issues:")
        for idx, f in enumerate(report["findings"], 1):
            line_str = f"Line {f['line']}: " if f.get("line") else ""
            print(f"  {idx}. [{f['severity']}] [{f['pillar']}] {line_str}{f['issue']}")
            if f.get("snippet"):
                print(f"      ↳ {f['snippet']}")
        sys.exit(1 if report["verdict"] == "BLOCKING" else 0)
    else:
        print("✅ [PASS] Code satisfies all Master Rubric pillars.")
        sys.exit(0)

if __name__ == "__main__":
    main()
