"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / benchmark_build_loop
- **Primary Entrypoints**: `run_benchmark`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[BENCHMARK]`
- **Witness Tests**: none declared
"""

import subprocess
import time
import json
import statistics
import sys
import shutil
from pathlib import Path

"""
Reproducible Developer Feedback & Build Loop Benchmark Harness
Measures realistic latencies across development loops:
1. Fast Verification / Doc Links Guard (Python AST & regex)
2. AST Symbol Graph Query (Rust binary lookup)
3. Sovereign Parity Guard (Python static analysis)
4. Incremental Rust Compile Check (cargo check)
"""

npm_cmd = shutil.which("npm") or ("npm.cmd" if sys.platform == "win32" else "npm")
python_cmd = sys.executable

BENCHMARK_TASKS = [
    {
        "id": "LOOP-DOC-GUARD",
        "name": "Doc & Link Integrity Guard",
        "command": [python_cmd, "execution/verify_doc_links.py", "."],
        "category": "Pre-Commit / Pre-Refactor",
        "claimed_target": "< 2.0s",
        "iterations": 3,
    },
    {
        "id": "LOOP-GRAPH-QUERY",
        "name": "AST Graph Symbol Blast Radius Lookup",
        "command": [npm_cmd, "run", "graph:lookup", "--", "--name", "AppState"],
        "category": "Agent Context Injection",
        "claimed_target": "< 1.5s",
        "iterations": 3,
    },
    {
        "id": "LOOP-PARITY-GUARD",
        "name": "Sovereign Parity Guard",
        "command": [python_cmd, "execution/parity_guard.py", "."],
        "category": "Pre-Commit Validation",
        "claimed_target": "< 4.0s",
        "iterations": 2,
    },
    {
        "id": "LOOP-RUST-CHECK",
        "name": "Backend Incremental Cargo Check",
        "command": ["cargo", "check", "--manifest-path", "server-rs/Cargo.toml"],
        "category": "Backend Verification",
        "claimed_target": "5.0s - 15.0s",
        "iterations": 2,
    },
]

def run_benchmark():
    print("================================================================================")
    print(" [BENCHMARK] TADPOLE OS BUILD & FEEDBACK LOOP REPRODUCIBLE BENCHMARK")
    print("================================================================================")
    results = []

    for task in BENCHMARK_TASKS:
        print(f"\n[*] Benchmarking [{task['id']}]: {task['name']} ({task['iterations']} runs)...")
        durations = []
        exit_codes = []

        for i in range(task["iterations"]):
            t0 = time.perf_counter()
            proc = subprocess.run(task["command"], shell=False, capture_output=True, text=True, encoding="utf-8", errors="replace")
            elapsed = time.perf_counter() - t0
            durations.append(elapsed)
            exit_codes.append(proc.returncode)
            status = "OK" if proc.returncode == 0 else f"ERR({proc.returncode})"
            print(f"    Run {i+1}: {elapsed:6.3f}s [{status}]")

        mean_dur = statistics.mean(durations)
        min_dur = min(durations)
        max_dur = max(durations)

        results.append({
            "id": task["id"],
            "name": task["name"],
            "category": task["category"],
            "target": task["claimed_target"],
            "mean_secs": round(mean_dur, 3),
            "min_secs": round(min_dur, 3),
            "max_secs": round(max_dur, 3),
            "status": "PASS" if all(c == 0 for c in exit_codes) else "FAIL"
        })

    print("\n" + "=" * 80)
    print(f"{'Loop ID':<18} | {'Category':<22} | {'Target':<14} | {'Measured (Mean)':<16} | {'Status'}")
    print("-" * 80)
    for r in results:
        print(f"{r['id']:<18} | {r['category']:<22} | {r['target']:<14} | {r['mean_secs']:>6.3f}s          | {r['status']}")
    print("=" * 80)

    # Export report to reports/benchmarks
    out_dir = Path("reports/benchmarks")
    out_dir.mkdir(parents=True, exist_ok=True)
    report_file = out_dir / "build_loop_benchmark.json"
    with open(report_file, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\n[+] Benchmark metrics saved to: {report_file}")

if __name__ == "__main__":
    run_benchmark()
