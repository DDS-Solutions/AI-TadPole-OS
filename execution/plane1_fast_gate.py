"""
@docs ARCHITECTURE:Core, ARCHITECTURE:Sandboxing

### AI Context Alignment
- **Subsystem**: Sovereign Engine / Plane 1 Verification
- **Primary Entrypoints**: `main`, `run_all_gates`, `run_gate`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.
- `[Structural]` Sub-15s total execution on warm caches across all parallel gates.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[Plane1FastGate]`
"""

import os
import re
import sys
import json
import time
import shutil
import argparse
import subprocess
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Dict, Any, List, Optional, Tuple

PROJECT_ROOT = Path(__file__).resolve().parent.parent

FAILURE_LINE_RE = re.compile(
    r"(\[FAIL\]|\bERROR\b|\berror\[|\bTS\d{4}:|AssertionError|\bFAIL\s|failed\b)",
    re.IGNORECASE,
)
WARNING_LINE_RE = re.compile(r"(^\s*warning(\[.*?\])?:|\[WARN\]|\bwarning\s+TS\d{4}:)", re.IGNORECASE)


def extract_failure_snippet(stdout: str, stderr: str, max_lines: int = 20) -> str:
    """Prioritize concrete failure lines before appending tail context."""
    combined_lines = [line for line in f"{stderr}\n{stdout}".splitlines() if line.strip()]
    if not combined_lines:
        return "No error snippet captured."

    priority = [line for line in combined_lines if FAILURE_LINE_RE.search(line)]
    if not priority:
        return "\n".join(combined_lines[-max_lines:])

    selected = priority[:max_lines]
    remaining = max_lines - len(selected)
    if remaining > 0:
        tail = [line for line in combined_lines[-remaining:] if line not in selected]
        if tail:
            selected.extend(["--- tail ---", *tail])
    return "\n".join(selected[: max_lines + 1])


def detect_warnings(stdout: str, stderr: str) -> List[str]:
    """Extract warning lines for --strict enforcement."""
    combined = f"{stderr}\n{stdout}".splitlines()
    return [line for line in combined if WARNING_LINE_RE.search(line)]


def is_windows() -> bool:
    return os.name == "nt"


def get_executable(cmd: str) -> str:
    """Resolve executable with .cmd extension on Windows if necessary."""
    if is_windows():
        cmd_path = shutil.which(f"{cmd}.cmd") or shutil.which(f"{cmd}.exe") or shutil.which(cmd)
        if cmd_path:
            return cmd_path
    return cmd


def run_gate(
    name: str,
    cmd: List[str],
    env_extra: Optional[Dict[str, str]] = None,
    cwd: Path = PROJECT_ROOT,
    strict: bool = False,
) -> Dict[str, Any]:
    """Execute a single verification gate and capture structured results."""
    start_time = time.time()
    env = os.environ.copy()
    if env_extra:
        env.update(env_extra)

    # Resolve executable on Windows
    resolved_cmd = list(cmd)
    resolved_cmd[0] = get_executable(resolved_cmd[0])

    try:
        proc = subprocess.run(
            resolved_cmd,
            cwd=str(cwd),
            env=env,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            shell=False,
        )
        duration_ms = int((time.time() - start_time) * 1000)
        output = (proc.stdout or "").strip()
        error = (proc.stderr or "").strip()
        warnings = detect_warnings(output, error) if strict else []
        success = proc.returncode == 0 and (not strict or len(warnings) == 0)

        snippet = ""
        if not success:
            if proc.returncode == 0 and warnings:
                snippet = "[STRICT] Warnings treated as failures:\n" + "\n".join(warnings[:20])
            else:
                snippet = extract_failure_snippet(output, error, max_lines=20)

        return {
            "name": name,
            "success": success,
            "duration_ms": duration_ms,
            "returncode": proc.returncode if proc.returncode != 0 else (2 if not success else 0),
            "warnings_count": len(warnings),
            "snippet": snippet,
            "raw_output": output if success else (error or output),
        }
    except Exception as e:
        duration_ms = int((time.time() - start_time) * 1000)
        return {
            "name": name,
            "success": False,
            "duration_ms": duration_ms,
            "returncode": -1,
            "warnings_count": 0,
            "snippet": f"Invocation error: {e}",
            "raw_output": str(e),
        }


def run_all_gates(
    skip_rust: bool = False,
    strict: bool = False,
    cwd: Optional[Path] = None,
) -> Tuple[bool, List[Dict[str, Any]], int]:
    """Execute all pre-flight gates in parallel."""
    t0 = time.time()
    target_cwd = cwd or Path.cwd()

    gates_to_run = [
        ("TypeScript Check", ["npx", "tsc", "-b"], None),
        ("TypeScript Tests", ["npx", "tsc", "-p", "tsconfig.test.json", "--noEmit"], None),
        ("ADG v2 Manifest", ["node", "tools/adg/adg.mjs", "verify", "src/"], None),
        ("Executable Invariants", ["npx", "vitest", "run", "tests/invariants/", "--no-isolate"], None),
        ("Header Parity Guard", ["python", "execution/parity_guard.py"], {"SKIP_RUST_COMPILE": "1"}),
    ]

    if not skip_rust:
        gates_to_run.append(
            ("Rust Fast Check", ["cargo", "check", "--manifest-path", "server-rs/Cargo.toml"], None)
        )

    results: List[Dict[str, Any]] = []

    # Run gates in parallel across a thread pool
    with ThreadPoolExecutor(max_workers=len(gates_to_run)) as executor:
        future_map = {
            executor.submit(run_gate, name, cmd, env, target_cwd, strict): name
            for name, cmd, env in gates_to_run
        }
        for future in as_completed(future_map):
            results.append(future.result())

    total_duration_ms = int((time.time() - t0) * 1000)

    # Sort results by original definition order
    order = [g[0] for g in gates_to_run]
    results.sort(key=lambda r: order.index(r["name"]) if r["name"] in order else 99)

    all_success = all(r["success"] for r in results)
    return all_success, results, total_duration_ms

def format_terminal_report(results: List[Dict[str, Any]], total_ms: float) -> str:
    """Format clean, human/LLM friendly terminal status table."""
    lines = [
        "",
        "==================================================================",
        f"  PLANE 1 PRE-FLIGHT FAST-GATE REPORT  (Total: {total_ms / 1000:.2f}s)",
        "==================================================================",
    ]
    
    failures = []
    for r in results:
        status_icon = "[PASS]" if r["success"] else "[FAIL]"
        sec_str = f"{r['duration_ms'] / 1000:.2f}s"
        lines.append(f"  {status_icon:<7} {r['name']:<26} ({sec_str})")
        if not r["success"]:
            failures.append(r)

    lines.append("------------------------------------------------------------------")
    
    if not failures:
        lines.append("  VERDICT: ALL GATES GREEN. Workspace is verified for Plane 1.")
    else:
        lines.append(f"  VERDICT: {len(failures)} GATE(S) FAILED. Inspect details below:")
        lines.append("------------------------------------------------------------------")
        for f in failures:
            lines.append(f"\n--- [FAIL] {f['name']} (Exit Code {f['returncode']}) ---")
            lines.append(f.get("snippet", "No error snippet captured."))
            
    lines.append("==================================================================")
    lines.append("")
    return "\n".join(lines)

def save_report_json(
    results: List[Dict[str, Any]],
    total_ms: int,
    all_success: bool,
    target_cwd: Optional[Path] = None,
):
    """Save machine-readable results to <cwd>/reports/plane1/fast_gate_results.json."""
    report_root = target_cwd or Path.cwd()
    reports_dir = report_root / "reports" / "plane1"
    reports_dir.mkdir(parents=True, exist_ok=True)
    report_file = reports_dir / "fast_gate_results.json"

    payload = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "all_passed": all_success,
        "total_duration_ms": total_ms,
        "gates": results,
    }
    with open(report_file, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)


def main():
    parser = argparse.ArgumentParser(description="Plane 1 Unified Fast-Gate Verification Runner")
    parser.add_argument("--skip-rust", action="store_true", help="Skip server-rs cargo check")
    parser.add_argument("--strict", action="store_true", help="Fail on any warning")
    parser.add_argument("--json", action="store_true", help="Output machine-readable JSON")
    args = parser.parse_args()

    target_cwd = Path.cwd()
    all_success, results, total_ms = run_all_gates(
        skip_rust=args.skip_rust,
        strict=args.strict,
        cwd=target_cwd,
    )
    save_report_json(results, total_ms, all_success, target_cwd=target_cwd)

    if args.json:
        payload = {
            "all_passed": all_success,
            "total_duration_ms": total_ms,
            "gates": results,
        }
        print(json.dumps(payload, indent=2))
    else:
        print("[Plane1FastGate] Pre-flight verification completed.")
        print(format_terminal_report(results, total_ms))

    sys.exit(0 if all_success else 1)

if __name__ == "__main__":
    main()

