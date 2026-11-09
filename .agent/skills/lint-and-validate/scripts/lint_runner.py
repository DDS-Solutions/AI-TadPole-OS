#!/usr/bin/env python3
"""
@docs ARCHITECTURE:SovereignArchitecture

### AI Context Alignment
- **Subsystem**: Agent Skills / lint-and-validate / lint_runner
- **Primary Entrypoints**: `detect_project_type`, `run_linter`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[lint_runner]`, `[DETECT]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import subprocess
import sys
import os
import json
import platform
import argparse
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Optional

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

def detect_project_type(project_path: Path) -> Dict[str, Any]:
    """Detect project ecosystems and available linters (Node.js, Rust, Python)."""
    result: Dict[str, Any] = {
        "types": [],
        "linters": []
    }

    # 1. Node.js / TypeScript
    package_json = project_path / "package.json"
    if package_json.exists():
        result["types"].append("node")
        try:
            pkg = json.loads(package_json.read_text(encoding='utf-8'))
            scripts = pkg.get("scripts", {})
            deps = {**pkg.get("dependencies", {}), **pkg.get("devDependencies", {})}

            if "lint" in scripts:
                result["linters"].append({"name": "npm lint", "cmd": ["npm", "run", "lint"]})
            elif "eslint" in deps:
                result["linters"].append({"name": "eslint", "cmd": ["npx", "eslint", "."]})

            if "format:check" in scripts or (project_path / "biome.json").exists():
                result["linters"].append({"name": "biome format", "cmd": ["npx", "biome", "format", "."]})

            if "typescript" in deps or (project_path / "tsconfig.json").exists():
                result["linters"].append({"name": "tsc", "cmd": ["npx", "tsc", "--noEmit"]})
        except Exception:
            pass

    # 2. Rust Backend
    cargo_toml = project_path / "Cargo.toml"
    server_rs_cargo = project_path / "server-rs" / "Cargo.toml"
    if cargo_toml.exists() or server_rs_cargo.exists():
        result["types"].append("rust")
        target_cargo = cargo_toml if cargo_toml.exists() else server_rs_cargo
        manifest_arg = ["--manifest-path", str(target_cargo.relative_to(project_path))] if target_cargo != cargo_toml else []
        result["linters"].append({"name": "cargo check", "cmd": ["cargo", "check"] + manifest_arg})
        result["linters"].append({"name": "cargo clippy", "cmd": ["cargo", "clippy"] + manifest_arg + ["--", "-D", "warnings"]})

    # 3. Python Automation
    if (project_path / "pyproject.toml").exists() or (project_path / "ruff.toml").exists() or (project_path / "requirements.txt").exists() or any(project_path.glob("*.py")):
        result["types"].append("python")
        result["linters"].append({"name": "ruff", "cmd": [sys.executable, "-m", "ruff", "check", "."]})

    return result

def run_linter(linter: Dict[str, Any], cwd: Path) -> Dict[str, Any]:
    """Run a single linter command deterministically and return output."""
    result = {
        "name": linter["name"],
        "passed": False,
        "output": "",
        "error": ""
    }

    try:
        cmd = list(linter["cmd"])

        # Windows compatibility for npm/npx
        if platform.system() == "Windows":
            if cmd[0] in ["npm", "npx"]:
                if not cmd[0].lower().endswith(".cmd"):
                    cmd[0] = f"{cmd[0]}.cmd"

        proc = subprocess.run(
            cmd,
            cwd=str(cwd),
            capture_output=True,
            text=True,
            encoding='utf-8',
            errors='replace',
            timeout=120,
            shell=platform.system() == "Windows"
        )

        result["output"] = proc.stdout[:2000] if proc.stdout else ""
        result["error"] = proc.stderr[:500] if proc.stderr else ""
        result["passed"] = proc.returncode == 0

    except FileNotFoundError:
        result["error"] = f"Command not found: {linter['cmd'][0]}"
    except subprocess.TimeoutExpired:
        result["error"] = "Timeout after 120s"
    except Exception as e:
        result["error"] = str(e)

    return result

def run_self_tests() -> bool:
    """Execute comprehensive in-memory unit tests."""
    print("[lint_runner] Executing self-test suite...")

    # Test 1: Node.js detection mock
    test_dir = Path(".tmp/test_lint_runner")
    test_dir.mkdir(parents=True, exist_ok=True)
    pkg_file = test_dir / "package.json"
    pkg_file.write_text(json.dumps({
        "name": "mock-pkg",
        "scripts": {"lint": "eslint ."},
        "devDependencies": {"typescript": "^5.0.0"}
    }), encoding='utf-8')

    res1 = detect_project_type(test_dir)
    assert "node" in res1["types"]
    assert any(l["name"] == "npm lint" for l in res1["linters"])
    assert any(l["name"] == "tsc" for l in res1["linters"])
    print("[lint_runner] [DETECT] Node.js & TypeScript linter detection verified.")

    # Test 2: Rust Cargo detection mock
    cargo_file = test_dir / "Cargo.toml"
    cargo_file.write_text('[package]\nname = "mock-rust"\nversion = "0.1.0"', encoding='utf-8')
    res2 = detect_project_type(test_dir)
    assert "rust" in res2["types"]
    assert any(l["name"] == "cargo check" for l in res2["linters"])
    print("[lint_runner] [DETECT] Rust Cargo linter detection verified.")

    # Clean up test dir
    if pkg_file.exists():
        pkg_file.unlink()
    if cargo_file.exists():
        cargo_file.unlink()
    if test_dir.exists():
        test_dir.rmdir()

    print("[lint_runner] [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Unified Sovereign Multi-Ecosystem Lint & Quality Runner."
    )
    parser.add_argument("path", nargs="?", default=".", help="Project root directory to lint.")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format.")
    parser.add_argument("--test", action="store_true", help="Run automated self-tests.")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print("[lint_runner] Initializing sovereign linting audit...")
    project_path = Path(args.path).resolve()

    if not project_path.exists():
        print(f"[lint_runner] [FAIL] Path does not exist: {project_path}")
        sys.exit(1)

    project_info = detect_project_type(project_path)
    linters = project_info.get("linters", [])

    if not linters:
        print("[lint_runner] [DETECT] No linters detected for target directory.")
        if args.json:
            print(json.dumps({"status": "no_linters", "types": project_info["types"], "passed": True}))
        sys.exit(0)

    print(f"[lint_runner] [DETECT] Detected ecosystems: {', '.join(project_info['types'])} ({len(linters)} checks)")

    results = []
    all_passed = True

    for linter in linters:
        print(f"Running: {linter['name']}...")
        result = run_linter(linter, project_path)
        results.append(result)

        if result["passed"]:
            print(f"  [lint_runner] [PASS] {linter['name']}")
        else:
            print(f"  [lint_runner] [FAIL] {linter['name']}")
            if result["error"]:
                print(f"  Error: {result['error']}")
            all_passed = False

    if args.json:
        report = {
            "project": str(project_path),
            "timestamp": datetime.now().isoformat(),
            "types": project_info["types"],
            "checks": results,
            "passed": all_passed
        }
        print(json.dumps(report, indent=2))
    else:
        print("\n" + "=" * 60)
        print("  LINT AUDIT SUMMARY")
        print("=" * 60)
        for r in results:
            icon = "[PASS]" if r["passed"] else "[FAIL]"
            print(f"  {icon} {r['name']}")
        if all_passed:
            print("[lint_runner] [PASS] All configured linters passed.")
        else:
            print("[lint_runner] [FAIL] Quality gate violations detected.")

    sys.exit(0 if all_passed else 1)

if __name__ == "__main__":
    main()
