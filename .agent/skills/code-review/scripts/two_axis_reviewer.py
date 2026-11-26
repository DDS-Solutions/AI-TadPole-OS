#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / code-review / two_axis_reviewer
- **Primary Entrypoints**: `audit_standards_axis`, `audit_spec_axis`, `evaluate_two_axis_report`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic review evaluation maintaining strict two-axis separation.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[two_axis_reviewer]`, `[TWO_AXIS]`, `[STANDARDS]`, `[SPEC]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import json
import re
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

FOWLER_SMELL_PATTERNS = [
    (r'\b(?:let|var|const)\s+([a-zA-Z])\s*[:=]', "Mysterious Name: Single-letter identifier lacks intent"),
    (r'\.unwrap\(\)', "Unhandled Rust unwrap() violating production error-handling standard"),
    (r'(?:\bas\s+any\b|:\s*any\b)', "TypeScript 'any' type escape hatch violating type safety"),
    (r'TODO:?\s*(?:clean|fix|refactor|remove)', "Technical Debt: Unresolved TODO comment in production code"),
]

def audit_standards_axis(diff_or_code: str) -> List[Dict[str, Any]]:
    """Scan diff or code for violations of repo conventions and Fowler code smells."""
    findings = []
    lines = diff_or_code.splitlines()

    for idx, line in enumerate(lines, 1):
        line_clean = line.strip()
        if line_clean.startswith(('//', '#', '/*', '*')):
            continue

        for pattern, desc in FOWLER_SMELL_PATTERNS:
            match = re.search(pattern, line_clean)
            if match:
                item = {
                    "line": idx,
                    "axis": "Standards",
                    "smell": desc,
                    "snippet": line_clean
                }
                findings.append(item)

    return findings

def audit_spec_axis(diff_or_code: str, requirements: Optional[List[str]] = None) -> List[Dict[str, Any]]:
    """Verify that declared requirements are implemented and reflected in the diff."""
    findings = []
    if not requirements:
        return findings

    diff_lower = diff_or_code.lower()

    for req in requirements:
        req_clean = req.strip()
        # Extract keywords from requirement (words >= 4 chars)
        keywords = [w.lower() for w in re.findall(r'\b[a-zA-Z0-9_-]{4,}\b', req_clean)]
        matched_keywords = [kw for kw in keywords if kw in diff_lower]

        if not matched_keywords and keywords:
            item = {
                "axis": "Spec",
                "requirement": req_clean,
                "status": "MISSING",
                "detail": f"No keywords from requirement '{req_clean}' found in diff"
            }
            findings.append(item)

    return findings

def evaluate_two_axis_report(report_text: str) -> Dict[str, Any]:
    """Validate that a code review report adheres to the Two-Axis separation contract."""
    report_lower = report_text.lower()
    violations = []

    has_standards_axis = "axis 1" in report_lower or "standards findings" in report_lower
    has_spec_axis = "axis 2" in report_lower or "spec findings" in report_lower

    if not has_standards_axis:
        violations.append("Report is missing dedicated 'Standards Axis' section.")

    if not has_spec_axis:
        violations.append("Report is missing dedicated 'Spec Axis' section.")

    has_standards_status = bool(re.search(r'standards status:\s*\[?(?:pass|fail)\]?', report_lower))
    has_spec_status = bool(re.search(r'spec status:\s*\[?(?:pass|fail)\]?', report_lower))

    if not has_standards_status:
        violations.append("Report summary missing 'Standards Status: [PASS/FAIL]' line.")

    if not has_spec_status:
        violations.append("Report summary missing 'Spec Status: [PASS/FAIL]' line.")

    return {
        "has_standards_axis": has_standards_axis,
        "has_spec_axis": has_spec_axis,
        "compliant": len(violations) == 0,
        "violations": violations
    }

def run_self_tests() -> bool:
    """Execute built-in self-tests verifying Two-Axis review logic."""
    print("🔬 [two_axis_reviewer] Running self-test suite...")

    # Test 1: Standards Axis audit
    sample_diff = """+ const x = 10;
+ let result = payload.unwrap();
+ const bad: any = true;
"""
    s_findings = audit_standards_axis(sample_diff)
    assert len(s_findings) >= 3, f"Test 1 Failed: Expected at least 3 standards findings, got: {s_findings}"

    # Test 2: Spec Axis audit
    spec_diff = "+ pub fn handle_oauth_callback() { /* login logic */ }"
    reqs = ["OAuth callback handler", "Scheduled token refresh cron"]
    spec_findings = audit_spec_axis(spec_diff, reqs)
    assert any("refresh" in f["requirement"].lower() for f in spec_findings), f"Test 2 Failed: Expected missing requirement, got: {spec_findings}"

    # Test 3: Compliant Two-Axis report
    valid_report = """# Code Review Report
## Axis 1: Standards Findings
- No critical code smells detected.

## Axis 2: Spec Findings
- [REQ-1] Implemented OAuth callback.

## Final Review Summary
- Standards Status: PASS - 0 findings
- Spec Status: PASS - All requirements met
"""
    res = evaluate_two_axis_report(valid_report)
    assert res["compliant"], f"Test 3 Failed: Compliant report failed evaluation: {res}"

    print("✅ [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic Two-Axis Code Review Auditor")
    parser.add_argument("--file", type=str, help="Path to code, diff, or review report to analyze")
    parser.add_argument("--spec-reqs", nargs="+", help="List of expected requirement keywords")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    content = ""
    target_name = "stdin"

    if args.file:
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"❌ [FAIL] File not found: {file_path}", file=sys.stderr)
            sys.exit(1)
        target_name = str(file_path)
        content = file_path.read_text(encoding='utf-8', errors='replace')
    elif not sys.stdin.isatty():
        content = sys.stdin.read()
    else:
        print("ℹ️  No input file or piped text provided. Running self-test suite.")
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print(f"🔍 [two_axis_reviewer] Evaluating Two-Axis review for: {target_name}")

    # Check if target is a review report or raw diff/code
    is_report = "standards findings" in content.lower() or "spec findings" in content.lower()

    if is_report:
        report_eval = evaluate_two_axis_report(content)
        print(f"📊 [TWO_AXIS] Separation: Standards={report_eval['has_standards_axis']}, Spec={report_eval['has_spec_axis']}")
        if args.json:
            print(json.dumps(report_eval, indent=2))
            sys.exit(0 if report_eval["compliant"] else 1)

        if not report_eval["compliant"]:
            print(f"❌ [FAIL] Two-Axis report contract violations:")
            for v in report_eval["violations"]:
                print(f"  ↳ {v}")
            sys.exit(1)
        else:
            print("✅ [PASS] Review report conforms strictly to Two-Axis separation standards.")
            sys.exit(0)
    else:
        standards_findings = audit_standards_axis(content)
        spec_findings = audit_spec_axis(content, args.spec_reqs)

        print(f"📡 [STANDARDS] Findings: {len(standards_findings)} | [SPEC] Missing: {len(spec_findings)}")

        result = {
            "target": target_name,
            "standards_findings": standards_findings,
            "spec_findings": spec_findings,
            "passed": len(standards_findings) == 0 and len(spec_findings) == 0
        }

        if args.json:
            print(json.dumps(result, indent=2))
            sys.exit(0 if result["passed"] else 1)

        if not result["passed"]:
            print(f"❌ [FAIL] Two-Axis Review detected issues:")
            for sf in standards_findings:
                print(f"  - [STANDARDS] Line {sf['line']}: {sf['smell']} ({sf['snippet']})")
            for sp in spec_findings:
                print(f"  - [SPEC] Missing Requirement: {sp['requirement']}")
            sys.exit(1)
        else:
            print("✅ [PASS] Diff passed both Standards and Spec verification.")
            sys.exit(0)

if __name__ == "__main__":
    main()
