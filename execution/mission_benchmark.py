"""Freeze mission conditions and score independently reviewed results offline.

@docs ARCHITECTURE:Infrastructure:Execution
### AI Assist Note
This tool never dispatches agents or transmits source. Semantic grades come from
an independent reviewer, never from a model's claimed completion status.
### 🔍 Debugging & Observability
JSON artifacts record comparable conditions, evidence hashes, and unscored cases.
"""

import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, ensure_ascii=False).encode()).hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def freeze(case, root=ROOT):
    for required in ("case_id", "purpose", "brief", "agents", "sources", "rubric", "history_policy"):
        if not case.get(required):
            raise ValueError(f"Missing case field: {required}")
    if case["purpose"] not in ("inward", "outward"):
        raise ValueError("purpose must be inward or outward")
    if any(not isinstance(v, (int, float)) or isinstance(v, bool) or not 0 < v < float("inf") for v in case["rubric"].values()):
        raise ValueError("Rubric weights must be positive finite numbers")
    sources = []
    for source in case["sources"]:
        path = (root / source).resolve()
        path.relative_to(root.resolve())
        sources.append({"path": source, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    conditions = {**case, "sources": sources}
    return {"schema_version": 1, "conditions": conditions, "case_fingerprint": digest(conditions)}


def score(frozen, run, grade=None, root=ROOT):
    if digest(frozen["conditions"]) != frozen["case_fingerprint"]:
        raise ValueError("Frozen conditions were changed")
    if run.get("case_fingerprint") != frozen["case_fingerprint"]:
        raise ValueError("Run conditions do not match the frozen case")
    required = ("handoffs_once", "reports_delivered", "scope_preserved", "budget_preserved")
    checks = run.get("execution_checks", {})
    execution_pass = all(checks.get(key) is True for key in required)
    result = {"case_fingerprint": frozen["case_fingerprint"], "case_id": frozen["conditions"]["case_id"],
              "execution_pass": execution_pass, "quality_status": "UNSCORED", "score": None,
              "maximum": sum(frozen["conditions"]["rubric"].values()), "evidence": []}
    if grade is None:
        return result
    if grade.get("case_fingerprint") != frozen["case_fingerprint"] or grade.get("reviewer_type") != "independent":
        raise ValueError("Grade must identify an independent review of this frozen case")
    if not grade.get("evidence_files"):
        raise ValueError("Independent grades require evidence files")
    for source in grade["evidence_files"]:
        path = (root / source).resolve()
        path.relative_to(root.resolve())
        result["evidence"].append({"path": source, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    points = grade.get("points", {})
    rubric = frozen["conditions"]["rubric"]
    if set(points) != set(rubric):
        raise ValueError("Every rubric criterion requires a grade")
    for key, maximum in rubric.items():
        value = points[key]
        if not isinstance(value, (int, float)) or isinstance(value, bool) or not 0 <= value <= maximum:
            raise ValueError(f"Invalid grade for {key}")
    result["score"] = sum(points.values())
    result["quality_status"] = "PASS" if execution_pass and result["score"] == result["maximum"] else "FAIL"
    return result


def compare(first, second):
    if first["case_fingerprint"] != second["case_fingerprint"]:
        raise ValueError("Different tasks, source revisions, models, temperatures, or history policies cannot be compared")
    return {"case_fingerprint": first["case_fingerprint"], "first": first, "second": second}


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    commands = parser.add_subparsers(dest="command", required=True)
    capture = commands.add_parser("freeze")
    capture.add_argument("--case", required=True)
    capture.add_argument("--out", required=True)
    evaluate = commands.add_parser("score")
    evaluate.add_argument("--frozen", required=True)
    evaluate.add_argument("--run", required=True)
    evaluate.add_argument("--grade")
    evaluate.add_argument("--out", required=True)
    comparison = commands.add_parser("compare")
    comparison.add_argument("first")
    comparison.add_argument("second")
    comparison.add_argument("--out", required=True)
    args = parser.parse_args()
    try:
        if args.command == "freeze":
            result = freeze(read_json(args.case))
        elif args.command == "score":
            result = score(read_json(args.frozen), read_json(args.run), read_json(args.grade) if args.grade else None)
        else:
            result = compare(read_json(args.first), read_json(args.second))
        Path(args.out).write_text(json.dumps(result, ensure_ascii=False, indent=2, allow_nan=False), encoding="utf-8")
        print(json.dumps({"status": result.get("quality_status", "RECORDED"), "artifact": args.out}))
    except (ValueError, KeyError, OSError, TypeError) as error:
        print(f"[mission_benchmark] {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
