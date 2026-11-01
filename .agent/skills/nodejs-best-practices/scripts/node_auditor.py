#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / nodejs-best-practices / node_auditor
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

SYNC_BLOCKING_PATTERNS = [
    (re.compile(r'\bfs\.(?:readFileSync|writeFileSync|appendFileSync|existsSync|statSync)\b'), "Blocking synchronous file system call detected (event loop hazard)"),
    (re.compile(r'crypto\.(?:pbkdf2Sync|randomBytesSync|scryptSync)\b'), "Blocking synchronous crypto call in event loop thread"),
    (re.compile(r'child_process\.execSync\b'), "Blocking execSync call (stalls Node.js event loop)"),
]

def audit_node_file(filepath: Path) -> List[Dict[str, Any]]:
    """Audit single Node.js / TypeScript file for event loop blockers and security hazards."""
    findings = []
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except Exception as e:
        return [{"line": 0, "rule": f"Cannot read file: {e}"}]

    # Skip test files and migration scripts from sync I/O restriction
    is_test_or_tool = bool(re.search(r'(\.test\.|\.spec\.|tools/|scripts/|migrations/)', str(filepath).replace('\\', '/')))

    for idx, line in enumerate(content.splitlines(), start=1):
        if not is_test_or_tool:
            for pattern, msg in SYNC_BLOCKING_PATTERNS:
                if pattern.search(line):
                    findings.append({
                        "file": filepath.name,
                        "line": idx,
                        "rule": msg,
                        "snippet": line.strip()[:80]
                    })

    return findings

def audit_node_directory(target_dir: Path) -> Dict[str, Any]:
    """Audit all JS/TS files in directory."""
    all_findings = []
    files_checked = 0

    for root, dirs, files in os.walk(target_dir):
        dirs[:] = [d for d in dirs if d not in ('node_modules', '.git', 'dist', 'build', '.next')]
        for f in files:
            if f.endswith(('.js', '.ts', '.mjs', '.cjs')) and not f.endswith('.d.ts'):
                p = Path(root) / f
                files_checked += 1
                all_findings.extend(audit_node_file(p))

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
    import fs from 'fs';
    export async function handler(req, res) {
        const data = fs.readFileSync('config.json', 'utf8');
        return res.json(JSON.parse(data));
    }
    """
    with tempfile.NamedTemporaryFile("w", suffix=".ts", delete=False) as f:
        f.write(bad_snippet)
        tmp_name = f.name

    try:
        findings = audit_node_file(Path(tmp_name))
        assert len(findings) >= 1, "Should detect readFileSync in service file"
    finally:
        os.remove(tmp_name)

    good_snippet = """
    import fs from 'fs/promises';
    export async function handler(req, res) {
        const data = await fs.readFile('config.json', 'utf8');
        return res.json(JSON.parse(data));
    }
    """
    with tempfile.NamedTemporaryFile("w", suffix=".ts", delete=False) as f:
        f.write(good_snippet)
        tmp_name2 = f.name

    try:
        findings2 = audit_node_file(Path(tmp_name2))
        assert len(findings2) == 0, f"Expected 0 violations for async code, got {len(findings2)}"
    finally:
        os.remove(tmp_name2)

    print("[PASS] node_auditor self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Node.js Event Loop & Security Auditor")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--audit-dir", type=str, help="Audit a directory of Node.js/TS files")
    parser.add_argument("--audit-file", type=str, help="Audit a single Node.js/TS file")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.audit_file:
        findings = audit_node_file(Path(args.audit_file))
        if findings:
            print(f"[FAIL] Found {len(findings)} violations:")
            for f in findings:
                print(f"  Line {f['line']}: {f['rule']}")
            sys.exit(1)
        else:
            print("[PASS] 0 violations found.")
            sys.exit(0)

    if args.audit_dir:
        report = audit_node_directory(Path(args.audit_dir))
        print(f"[AUDIT] Checked {report['files_checked']} files, {report['violations']} violations.")
        sys.exit(0 if report["compliant"] else 1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
