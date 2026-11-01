"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Benchmarks & Verification / mission_bench/run
- **Primary Entrypoints**: `validate_scenario`, `init_results_db`, `load_scenarios`, `run_scenario_dry`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic YAML scenario validation against benchmark schema.
- `[Structural]` SQLite persistence for all run telemetry and per-criterion evaluations.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared

Mission Bench — Blind-Judge Benchmarking Rig

Evaluates mission quality by running realistic scenarios against
the Sovereign Engine and grading results with a blind LLM judge.

Usage:
  python tests/mission_bench/run.py                  # Run all benchmarks
  python tests/mission_bench/run.py --dry-run        # Validate schemas only
  python tests/mission_bench/run.py --scenario greeting_agent

Results are logged to .tmp/mission_bench/results.db (SQLite).
"""

import argparse
import json
import os
import sqlite3
import sys
import time
import yaml
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

# Fix Windows console encoding for emoji output
if sys.platform == "win32" and hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")


# ── Configuration ─────────────────────────────────────────────

BENCH_DIR = Path(__file__).parent
SCENARIOS_DIR = BENCH_DIR / "scenarios"
RESULTS_DIR = Path(os.environ.get("TADPOLE_WORKSPACE", str(BENCH_DIR.parent.parent))) / ".tmp" / "mission_bench"


# ── Schema Validation ─────────────────────────────────────────

REQUIRED_CRITERIA_FIELDS = {"id", "name", "description", "mission_prompt", "criteria"}
REQUIRED_CRITERION_FIELDS = {"id", "description", "weight"}


def validate_scenario(scenario_path: Path) -> List[str]:
    """Validate a scenario YAML file against the schema. Returns list of errors."""
    errors = []
    try:
        with open(scenario_path, "r", encoding="utf-8") as f:
            data = yaml.safe_load(f)
    except Exception as e:
        return [f"YAML parse error: {e}"]

    if not isinstance(data, dict):
        return ["Root element must be a mapping"]

    missing = REQUIRED_CRITERIA_FIELDS - set(data.keys())
    if missing:
        errors.append(f"Missing required fields: {missing}")

    if "criteria" in data:
        if not isinstance(data["criteria"], list):
            errors.append("'criteria' must be a list")
        else:
            for i, c in enumerate(data["criteria"]):
                if not isinstance(c, dict):
                    errors.append(f"criteria[{i}] must be a mapping")
                    continue
                c_missing = REQUIRED_CRITERION_FIELDS - set(c.keys())
                if c_missing:
                    errors.append(f"criteria[{i}] missing fields: {c_missing}")
                if "weight" in c and not isinstance(c["weight"], (int, float)):
                    errors.append(f"criteria[{i}].weight must be a number")

    if "expected_tools" in data and not isinstance(data["expected_tools"], list):
        errors.append("'expected_tools' must be a list if present")

    if "max_turns" in data and not isinstance(data["max_turns"], int):
        errors.append("'max_turns' must be an integer if present")

    if "timeout_seconds" in data and not isinstance(data["timeout_seconds"], (int, float)):
        errors.append("'timeout_seconds' must be a number if present")

    return errors


# ── Results Database ──────────────────────────────────────────

def init_results_db(db_path: Path) -> sqlite3.Connection:
    """Initialize the SQLite results database."""
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.execute("""
        CREATE TABLE IF NOT EXISTS runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scenario_id TEXT NOT NULL,
            scenario_name TEXT NOT NULL,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            status TEXT DEFAULT 'pending',
            total_turns INTEGER DEFAULT 0,
            total_tool_calls INTEGER DEFAULT 0,
            total_tokens INTEGER DEFAULT 0,
            total_cost_usd REAL DEFAULT 0.0,
            duration_seconds REAL DEFAULT 0.0,
            judge_score REAL,
            judge_pass INTEGER,
            judge_rationale TEXT,
            raw_output TEXT,
            error TEXT
        )
    """)
    conn.execute("""
        CREATE TABLE IF NOT EXISTS criterion_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL REFERENCES runs(id),
            criterion_id TEXT NOT NULL,
            criterion_description TEXT,
            weight REAL DEFAULT 1.0,
            passed INTEGER DEFAULT 0,
            score REAL DEFAULT 0.0,
            rationale TEXT
        )
    """)
    conn.commit()
    return conn


# ── Scenario Runner ───────────────────────────────────────────

def load_scenarios(scenario_filter: Optional[str] = None) -> List[Dict[str, Any]]:
    """Load all scenarios from the scenarios/ directory."""
    scenarios = []
    if not SCENARIOS_DIR.exists():
        print(f"⚠️  No scenarios directory found at {SCENARIOS_DIR}")
        return scenarios

    for f in sorted(SCENARIOS_DIR.glob("*.yaml")):
        with open(f, "r", encoding="utf-8") as fh:
            data = yaml.safe_load(fh)
            data["_path"] = str(f)
            if scenario_filter and data.get("id") != scenario_filter:
                continue
            scenarios.append(data)

    return scenarios


def run_scenario_dry(scenario: Dict[str, Any]) -> Dict[str, Any]:
    """Dry run — validate schema only, no execution."""
    errors = validate_scenario(Path(scenario["_path"]))
    return {
        "scenario_id": scenario.get("id", "unknown"),
        "scenario_name": scenario.get("name", "unknown"),
        "status": "valid" if not errors else "invalid",
        "errors": errors,
        "criteria_count": len(scenario.get("criteria", [])),
        "max_turns": scenario.get("max_turns", "unlimited"),
    }


# ── CLI Entry Point ───────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description="TadpoleOS Mission Benchmarking Rig — Blind-Judge Evaluation"
    )
    parser.add_argument(
        "--dry-run", action="store_true",
        help="Validate scenario schemas without running missions"
    )
    parser.add_argument(
        "--scenario", type=str, default=None,
        help="Run a specific scenario by ID"
    )
    parser.add_argument(
        "--db", type=str, default=str(RESULTS_DIR / "results.db"),
        help="Path to results SQLite database"
    )
    args = parser.parse_args()

    scenarios = load_scenarios(args.scenario)

    if not scenarios:
        print("❌ No scenarios found. Create YAML files in tests/mission_bench/scenarios/")
        sys.exit(1)

    print(f"📋 Found {len(scenarios)} scenario(s)\n")

    if args.dry_run:
        all_valid = True
        for s in scenarios:
            result = run_scenario_dry(s)
            status_icon = "✅" if result["status"] == "valid" else "❌"
            print(f"  {status_icon} {result['scenario_id']}: {result['scenario_name']}")
            print(f"     Criteria: {result['criteria_count']}, Max turns: {result['max_turns']}")
            if result["errors"]:
                all_valid = False
                for e in result["errors"]:
                    print(f"     ⚠️  {e}")
            print()

        if all_valid:
            print("✅ All scenarios valid")
            sys.exit(0)
        else:
            print("❌ Some scenarios have validation errors")
            sys.exit(1)

    # Full execution mode
    db = init_results_db(Path(args.db))
    print(f"📊 Results DB: {args.db}\n")

    for scenario in scenarios:
        scenario_id = scenario.get("id", "unknown")
        print(f"🚀 Running: {scenario_id} — {scenario.get('name', '')}")

        started_at = datetime.now(timezone.utc).isoformat()
        start_time = time.monotonic()

        # Insert pending run
        cursor = db.execute(
            "INSERT INTO runs (scenario_id, scenario_name, started_at, status) VALUES (?, ?, ?, ?)",
            (scenario_id, scenario.get("name", ""), started_at, "pending")
        )
        run_id = cursor.lastrowid
        db.commit()

        try:
            # TODO: Wire to actual mission execution via HTTP API or IPC bridge
            # For now, mark as skipped in non-dry-run mode
            elapsed = time.monotonic() - start_time
            db.execute(
                "UPDATE runs SET status = ?, finished_at = ?, duration_seconds = ?, error = ? WHERE id = ?",
                (
                    "skipped",
                    datetime.now(timezone.utc).isoformat(),
                    elapsed,
                    "Mission execution not yet wired — use --dry-run to validate schemas",
                    run_id,
                )
            )
            db.commit()
            print(f"   ⏭️  Skipped (execution not yet wired) — {elapsed:.2f}s\n")

        except Exception as e:
            elapsed = time.monotonic() - start_time
            db.execute(
                "UPDATE runs SET status = ?, finished_at = ?, duration_seconds = ?, error = ? WHERE id = ?",
                ("error", datetime.now(timezone.utc).isoformat(), elapsed, str(e), run_id)
            )
            db.commit()
            print(f"   ❌ Error: {e} — {elapsed:.2f}s\n")

    db.close()
    print("✅ Benchmark run complete")


if __name__ == "__main__":
    main()
