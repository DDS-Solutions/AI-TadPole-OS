#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / tauri-pro / tauri_auditor
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import re
import argparse
from pathlib import Path

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')


def audit_rust_commands(rust_code: str) -> list[dict]:
    findings = []
    # Find all functions annotated with #[tauri::command]
    cmd_pattern = re.compile(r"#\[tauri::command(?:|\(.*?\))\]\s*(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\((.*?)\)(?:\s*->\s*([^{]+))?\s*\{", re.DOTALL)
    
    for match in cmd_pattern.finditer(rust_code):
        fn_name = match.group(1)
        ret_type = match.group(3).strip() if match.group(3) else "()"
        
        issues = []
        if not ret_type.startswith("Result<") and ret_type != "()":
            issues.append(f"Command '{fn_name}' returns '{ret_type}' instead of Result<T, E>")
            
        findings.append({
            "command": fn_name,
            "return_type": ret_type,
            "issues": issues,
            "valid": len(issues) == 0
        })

    return findings


def run_self_tests():
    sample_rust = """
    #[tauri::command]
    pub async fn fetch_nodes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
        Ok(vec!["node1".to_string()])
    }

    #[tauri::command]
    fn bad_command() -> Vec<String> {
        vec![]
    }
    """
    res = audit_rust_commands(sample_rust)
    assert len(res) == 2, f"Expected 2 commands, found {len(res)}"
    assert res[0]["valid"] is True
    assert res[1]["valid"] is False
    assert "instead of Result" in res[1]["issues"][0]
    print("[PASS] tauri_auditor self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Tauri IPC & Command Auditor")
    parser.add_argument("path", nargs="?", default="server-rs/src", help="Path to Rust source directory")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    target_path = Path(args.path)
    if not target_path.exists():
        print(f"Path '{target_path}' not found.", file=sys.stderr)
        sys.exit(1)

    total_cmds = 0
    total_issues = 0

    for rs_file in target_path.rglob("*.rs"):
        content = rs_file.read_text(encoding="utf-8", errors="ignore")
        cmds = audit_rust_commands(content)
        for c in cmds:
            total_cmds += 1
            if not c["valid"]:
                total_issues += 1
                print(f"[FAIL] {rs_file.name} :: {c['command']} -> {', '.join(c['issues'])}")

    if total_issues == 0:
        print(f"[OK] Audited {total_cmds} Tauri commands across '{target_path}'. All comply with Result return types.")
    else:
        print(f"[WARN] Found {total_issues} issues across {total_cmds} commands.")
        sys.exit(1)


if __name__ == "__main__":
    main()
