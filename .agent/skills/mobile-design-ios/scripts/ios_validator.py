#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / mobile-design-ios / ios_validator
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

ANTI_PATTERNS = [
    (re.compile(r'\bclass\s+\w+\s*:\s*ObservableObject\b'), "Legacy 'ObservableObject' used instead of modern '@Observable' macro (iOS 17+)"),
    (re.compile(r'@Published\s+var\b'), "Legacy '@Published' property wrapper used instead of '@Observable' field tracking"),
    (re.compile(r'UserDefaults\.standard\.set\([^,]+,\s*forKey:\s*"[^"]*(?:token|auth|secret|password|key)[^"]*"', re.I), "Insecure secret persistence in UserDefaults; use iOS Keychain"),
    (re.compile(r'\.frame\s*\(\s*(?:width|height)\s*:\s*([0-3]\d)\b'), "Touch target dimension < 44pt violating Apple HIG minimum hit target")
]

def audit_swift_file(filepath: Path) -> List[Dict[str, Any]]:
    """Audit single Swift file for HIG violations and anti-patterns."""
    findings = []
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except Exception as e:
        return [{"line": 0, "rule": f"Cannot read file: {e}"}]

    for idx, line in enumerate(content.splitlines(), start=1):
        for pattern, rule_msg in ANTI_PATTERNS:
            if pattern.search(line):
                findings.append({
                    "file": str(filepath.name),
                    "line": idx,
                    "rule": rule_msg,
                    "snippet": line.strip()[:80]
                })

    return findings

def audit_swift_directory(target_dir: Path) -> Dict[str, Any]:
    """Audit all Swift files in a directory."""
    all_findings = []
    files_checked = 0

    for swift_file in target_dir.rglob("*.swift"):
        files_checked += 1
        findings = audit_swift_file(swift_file)
        all_findings.extend(findings)

    return {
        "files_checked": files_checked,
        "violations": len(all_findings),
        "findings": all_findings,
        "compliant": len(all_findings) == 0
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    import tempfile

    test_code_bad = """
    import SwiftUI

    class UserViewModel: ObservableObject {
        @Published var token: String = ""
        
        func save() {
            UserDefaults.standard.set(token, forKey: "auth_token")
        }
    }
    """
    with tempfile.NamedTemporaryFile("w", suffix=".swift", delete=False) as f:
        f.write(test_code_bad)
        tmp_name = f.name

    try:
        findings = audit_swift_file(Path(tmp_name))
        assert len(findings) >= 3, f"Expected at least 3 violations, found {len(findings)}"
    finally:
        os.remove(tmp_name)

    test_code_good = """
    import SwiftUI

    @Observable
    class UserViewModel {
        var username: String = ""
    }
    """
    with tempfile.NamedTemporaryFile("w", suffix=".swift", delete=False) as f:
        f.write(test_code_good)
        tmp_name2 = f.name

    try:
        findings2 = audit_swift_file(Path(tmp_name2))
        assert len(findings2) == 0, f"Expected 0 violations for clean code, found {len(findings2)}"
    finally:
        os.remove(tmp_name2)

    print("[PASS] ios_validator self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Apple HIG & SwiftUI Static Auditor")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--audit-dir", type=str, help="Audit a directory of Swift files")
    parser.add_argument("--audit-file", type=str, help="Audit a single Swift file")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.audit_file:
        findings = audit_swift_file(Path(args.audit_file))
        if findings:
            print(f"[FAIL] Found {len(findings)} violations:")
            for f in findings:
                print(f"  Line {f['line']}: {f['rule']}")
            sys.exit(1)
        else:
            print("[PASS] 0 violations found.")
            sys.exit(0)

    if args.audit_dir:
        report = audit_swift_directory(Path(args.audit_dir))
        print(f"[AUDIT] Checked {report['files_checked']} files, {report['violations']} violations.")
        sys.exit(0 if report["compliant"] else 1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
