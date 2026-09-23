"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Benchmarks & Verification / mission_bench/judge
- **Primary Entrypoints**: `build_judge_prompt`, `call_local_judge`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Impartial evaluation against YAML criteria with JSON response schema.
- `[Structural]` Safe fallback to heuristic scoring when local LLM endpoint is unreachable.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared

Blind-Judge Evaluator for Mission Bench results.

Reads mission outputs from the results DB and evaluates them against
criteria using a configurable LLM judge (supports local models via
Ollama/LM Studio or cloud APIs).

Usage:
    python tests/mission_bench/judge.py                        # Judge all pending runs
    python tests/mission_bench/judge.py --run-id 42            # Judge specific run
    python tests/mission_bench/judge.py --model ollama/gemma2  # Use specific model
    python tests/mission_bench/judge.py --dry-run              # Preview prompts only
"""

import argparse
import json
import os
import sqlite3
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional


BENCH_DIR = Path(__file__).parent
RESULTS_DIR = Path(os.environ.get("TADPOLE_WORKSPACE", str(BENCH_DIR.parent.parent))) / ".tmp" / "mission_bench"

# ── Judge Prompt Template ─────────────────────────────────────

JUDGE_SYSTEM_PROMPT = """You are a strict, impartial judge evaluating AI agent mission outputs.
You will receive:
1. The original mission prompt (what the agent was asked to do)
2. The agent's output (what the agent produced)
3. A list of evaluation criteria with weights

For EACH criterion, you must:
- Determine if the criterion was MET (1) or NOT MET (0)
- Assign a score from 0.0 to 1.0
- Provide a brief rationale (1-2 sentences)

Respond in JSON format:
{
  "overall_pass": true/false,
  "overall_score": 0.0-1.0,
  "overall_rationale": "Brief summary",
  "criteria": [
    {
      "id": "criterion_id",
      "passed": true/false,
      "score": 0.0-1.0,
      "rationale": "Brief rationale"
    }
  ]
}

Be strict. If the output is vague, incomplete, or contains hallucinations, score accordingly.
Do not be lenient. The goal is honest evaluation."""


def build_judge_prompt(
    mission_prompt: str,
    agent_output: str,
    criteria: List[Dict[str, Any]],
) -> str:
    """Build the evaluation prompt for the LLM judge."""
    criteria_text = "\n".join([
        f"  {i+1}. [{c['id']}] (weight: {c.get('weight', 1.0)}): {c['description']}"
        for i, c in enumerate(criteria)
    ])

    return f"""## Mission Prompt
{mission_prompt}

## Agent Output
{agent_output}

## Evaluation Criteria
{criteria_text}

Evaluate the agent output against ALL criteria above. Respond with JSON only."""


# ── Judge Execution ───────────────────────────────────────────

def call_local_judge(
    system_prompt: str,
    user_prompt: str,
    model: str = "ollama/gemma2",
    base_url: Optional[str] = None,
) -> Dict[str, Any]:
    """
    Call a local LLM for judging. Supports Ollama and LM Studio.
    
    Falls back to a simple heuristic if no model is available.
    """
    import urllib.request
    import urllib.error

    # Determine API endpoint
    if base_url is None:
        if model.startswith("ollama/"):
            base_url = "http://localhost:11434/api/chat"
            model_name = model.replace("ollama/", "")
        else:
            base_url = "http://localhost:1234/v1/chat/completions"
            model_name = model

    if model.startswith("ollama/"):
        payload = {
            "model": model_name,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            "stream": False,
            "format": "json",
        }
    else:
        payload = {
            "model": model_name,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            "temperature": 0.1,
            "response_format": {"type": "json_object"},
        }

    try:
        req = urllib.request.Request(
            base_url,
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            result = json.loads(resp.read().decode("utf-8"))

        if model.startswith("ollama/"):
            content = result.get("message", {}).get("content", "{}")
        else:
            content = result.get("choices", [{}])[0].get("message", {}).get("content", "{}")

        return json.loads(content)

    except (urllib.error.URLError, ConnectionRefusedError) as e:
        print(f"  ⚠️  Cannot reach local LLM ({e}). Using heuristic fallback.")
        return _heuristic_judge(user_prompt)
    except json.JSONDecodeError as e:
        print(f"  ⚠️  LLM returned invalid JSON ({e}). Using heuristic fallback.")
        return _heuristic_judge(user_prompt)


def _heuristic_judge(prompt: str) -> Dict[str, Any]:
    """Simple heuristic fallback when no LLM is available."""
    return {
        "overall_pass": False,
        "overall_score": 0.0,
        "overall_rationale": "No LLM judge available — heuristic fallback. Run with a local model for real evaluation.",
        "criteria": [],
    }


# ── CLI Entry Point ───────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Blind-Judge Mission Evaluator")
    parser.add_argument("--run-id", type=int, help="Judge a specific run by ID")
    parser.add_argument("--model", type=str, default="ollama/gemma2", help="LLM model for judging")
    parser.add_argument("--db", type=str, default=str(RESULTS_DIR / "results.db"))
    parser.add_argument("--dry-run", action="store_true", help="Preview judge prompts without calling LLM")
    args = parser.parse_args()

    db_path = Path(args.db)
    if not db_path.exists():
        print(f"❌ No results database at {db_path}. Run benchmarks first.")
        sys.exit(1)

    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row

    # Find runs to judge
    if args.run_id:
        runs = conn.execute("SELECT * FROM runs WHERE id = ?", (args.run_id,)).fetchall()
    else:
        runs = conn.execute("SELECT * FROM runs WHERE judge_score IS NULL AND raw_output IS NOT NULL").fetchall()

    if not runs:
        print("ℹ️  No runs pending judgment. Execute benchmarks first or specify --run-id.")
        sys.exit(0)

    print(f"⚖️  Judging {len(runs)} run(s) with model: {args.model}\n")

    for run in runs:
        scenario_id = run["scenario_id"]
        print(f"  📝 Judging run #{run['id']}: {scenario_id}")

        # Load scenario criteria
        scenario_path = BENCH_DIR / "scenarios" / f"{scenario_id}.yaml"
        if not scenario_path.exists():
            print(f"     ⚠️  Scenario file not found: {scenario_path}")
            continue

        import yaml
        with open(scenario_path, "r") as f:
            scenario = yaml.safe_load(f)

        prompt = build_judge_prompt(
            mission_prompt=scenario.get("mission_prompt", ""),
            agent_output=run["raw_output"] or "",
            criteria=scenario.get("criteria", []),
        )

        if args.dry_run:
            print(f"     [DRY RUN] Prompt length: {len(prompt)} chars")
            print(f"     Criteria: {len(scenario.get('criteria', []))}")
            continue

        result = call_local_judge(JUDGE_SYSTEM_PROMPT, prompt, model=args.model)

        # Store results
        overall_pass = 1 if result.get("overall_pass") else 0
        overall_score = result.get("overall_score", 0.0)
        rationale = result.get("overall_rationale", "")

        conn.execute(
            "UPDATE runs SET judge_score = ?, judge_pass = ?, judge_rationale = ? WHERE id = ?",
            (overall_score, overall_pass, rationale, run["id"])
        )

        for cr in result.get("criteria", []):
            conn.execute(
                "INSERT INTO criterion_results (run_id, criterion_id, passed, score, rationale) VALUES (?, ?, ?, ?, ?)",
                (run["id"], cr.get("id", ""), 1 if cr.get("passed") else 0, cr.get("score", 0.0), cr.get("rationale", ""))
            )

        conn.commit()

        status = "✅ PASS" if overall_pass else "❌ FAIL"
        print(f"     {status} — Score: {overall_score:.2f} — {rationale[:80]}")

    conn.close()
    print("\n✅ Judgment complete")


if __name__ == "__main__":
    main()
