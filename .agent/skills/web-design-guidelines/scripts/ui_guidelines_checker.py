#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / web-design-guidelines / ui_guidelines_checker
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import os
import re
import argparse
from pathlib import Path
from typing import Dict, List, Any

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

UI_VIOLATIONS = [
    (re.compile(r'outline-none(?!\s+focus(?:-visible)?:)'), "Removing focus outline without providing focus-visible ring (violates WCAG 2.4.7)"),
    (re.compile(r'<div[^>]*\sonClick=[^>]*>(?![^>]*\brole=["\']button["\'])'), "Interactive div with onClick lacking role='button' (accessibility hazard)"),
    (re.compile(r'user-scalable\s*=\s*no'), "user-scalable=no disables browser pinch-to-zoom (violates WCAG 1.4.4)")
]

def audit_ui_file(filepath: Path) -> List[Dict[str, Any]]:
    """Audit single UI source file for Web Interface Guidelines violations."""
    findings = []
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except Exception as e:
        return [{"line": 0, "rule": f"Cannot read file: {e}"}]

    lines = content.splitlines()

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith(('//', '/*', '*')):
            continue

        for pattern, rule_msg in UI_VIOLATIONS:
            if pattern.search(line):
                findings.append({
                    "file": filepath.name,
                    "line": idx,
                    "rule": rule_msg,
                    "snippet": stripped[:80]
                })

    return findings

def audit_ui_directory(target_dir: Path) -> Dict[str, Any]:
    """Scan all frontend component files in directory."""
    all_findings = []
    files_checked = 0

    for root, dirs, files in os.walk(target_dir):
        dirs[:] = [d for d in dirs if d not in ('node_modules', '.git', 'dist', 'build', '.next')]
        for f in files:
            if f.endswith(('.tsx', '.jsx', '.html', '.css')):
                p = Path(root) / f
                files_checked += 1
                for finding in audit_ui_file(p):
                    finding["file"] = str(p.relative_to(target_dir))
                    all_findings.append(finding)

    return {
        "files_checked": files_checked,
        "violations": len(all_findings),
        "findings": all_findings,
        "compliant": len(all_findings) == 0
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    import tempfile

    bad_snippet = """
    export function BadModal() {
        return (
            <div className="outline-none">
                <div onClick={() => closeModal()}>Close</div>
            </div>
        );
    }
    """
    with tempfile.NamedTemporaryFile("w", suffix=".tsx", delete=False) as f:
        f.write(bad_snippet)
        tmp_name = f.name

    try:
        findings = audit_ui_file(Path(tmp_name))
        assert len(findings) >= 2, f"Expected 2 violations, got {len(findings)}"
    finally:
        os.remove(tmp_name)

    good_snippet = """
    export function GoodModal() {
        return (
            <div className="outline-none focus-visible:ring-2">
                <button type="button" onClick={() => closeModal()}>Close</button>
            </div>
        );
    }
    """
    with tempfile.NamedTemporaryFile("w", suffix=".tsx", delete=False) as f:
        f.write(good_snippet)
        tmp_name2 = f.name

    try:
        findings2 = audit_ui_file(Path(tmp_name2))
        assert len(findings2) == 0, f"Expected 0 violations for clean code, got {len(findings2)}"
    finally:
        os.remove(tmp_name2)

    print("[PASS] ui_guidelines_checker self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Web Interface Guidelines & Accessibility Checker")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to file to audit")
    parser.add_argument("--audit-dir", type=str, help="Path to directory of UI components")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        findings = audit_ui_file(p)
        if findings:
            print(f"[FAIL] Found {len(findings)} UI guideline violations in {p.name}:")
            for f in findings:
                print(f"  Line {f['line']}: {f['rule']}")
            sys.exit(1)
        else:
            print(f"[PASS] {p.name} conforms to Web Interface Guidelines.")
            sys.exit(0)

    if args.audit_dir:
        report = audit_ui_directory(Path(args.audit_dir))
        print(f"[AUDIT] Checked {report['files_checked']} files, {report['violations']} violations.")
        sys.exit(0 if report["compliant"] else 1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
