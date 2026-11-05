#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / rust-pro / rust_auditor
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

RUST_HAZARDS = [
    (re.compile(r'\bstd::thread::sleep\b'), "Blocking std::thread::sleep inside async code; use tokio::time::sleep"),
    (re.compile(r'\bstd::fs::(?:read|write|remove|copy)\b'), "Blocking std::fs inside async handler; use tokio::fs or spawn_blocking"),
    (re.compile(r'\.unwrap\(\)'), "Raw .unwrap() in production path; return Result with ? or map_err")
]

def audit_rust_file(filepath: Path) -> List[Dict[str, Any]]:
    """Audit single .rs file for blocking calls and unwrap panics."""
    findings = []
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except Exception as e:
        return [{"line": 0, "rule": f"Cannot read file: {e}"}]

    # Skip test modules and benches from unwrap check
    is_test_file = bool(re.search(r'(tests/|benches/|\.test\.rs$)', str(filepath).replace('\\', '/')))

    lines = content.splitlines()
    in_test_mod = False

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith('//'):
            continue

        if '#[cfg(test)]' in stripped or 'mod test' in stripped:
            in_test_mod = True

        for pattern, msg in RUST_HAZARDS:
            if pattern.search(line):
                # Allow unwrap inside test modules
                if "unwrap" in msg and (is_test_file or in_test_mod):
                    continue
                findings.append({
                    "file": filepath.name,
                    "line": idx,
                    "rule": msg,
                    "snippet": stripped[:80]
                })

    return findings

def audit_rust_directory(target_dir: Path) -> Dict[str, Any]:
    """Audit all .rs files in directory."""
    all_findings = []
    files_checked = 0

    for root, dirs, files in os.walk(target_dir):
        dirs[:] = [d for d in dirs if d not in ('target', '.git', 'node_modules')]
        for f in files:
            if f.endswith('.rs'):
                p = Path(root) / f
                files_checked += 1
                for finding in audit_rust_file(p):
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

    bad_code = """
    pub async fn handle_request() {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let val = Some(42).unwrap();
    }
    """
    with tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False) as f:
        f.write(bad_code)
        tmp_name = f.name

    try:
        findings = audit_rust_file(Path(tmp_name))
        assert len(findings) >= 2, f"Expected at least 2 violations, got {len(findings)}"
    finally:
        os.remove(tmp_name)

    good_code = """
    use tokio::time::{sleep, Duration};

    pub async fn handle_request() -> Result<i32, AppError> {
        sleep(Duration::from_millis(100)).await;
        Ok(42)
    }
    """
    with tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False) as f:
        f.write(good_code)
        tmp_name2 = f.name

    try:
        findings2 = audit_rust_file(Path(tmp_name2))
        assert len(findings2) == 0, f"Expected 0 violations for clean code, got {len(findings2)}"
    finally:
        os.remove(tmp_name2)

    print("[PASS] rust_auditor self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Rust Systems & Async Hazard Auditor")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to .rs file to audit")
    parser.add_argument("--audit-dir", type=str, help="Path to directory of Rust files")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        findings = audit_rust_file(p)
        if findings:
            print(f"[FAIL] Found {len(findings)} violations in {p.name}:")
            for f in findings:
                print(f"  Line {f['line']}: {f['rule']}")
            sys.exit(1)
        else:
            print(f"[PASS] {p.name} clean.")
            sys.exit(0)

    if args.audit_dir:
        report = audit_rust_directory(Path(args.audit_dir))
        print(f"[AUDIT] Checked {report['files_checked']} files, {report['violations']} violations.")
        sys.exit(0 if report["compliant"] else 1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
