# Repeatable mission evaluation

`execution/mission_benchmark.py` records and evaluates mission evidence offline. It never dispatches agents, changes model settings, or transmits source. Runtime acceptance and the correctness of a finding are separate results.

## Freeze before dispatch

Create a local case file containing `case_id`, `purpose` (`inward` or `outward`), `brief`, `agents`, `sources`, `rubric`, and `history_policy`.

```json
{
  "case_id": "completion-audit-1",
  "purpose": "inward",
  "brief": "Inspect the supplied execution code for one behavioral defect. Provide a trigger, source evidence, correction, and regression idea. Untested claims remain candidates.",
  "agents": [
    {"id":"1", "model":"gemma4:31b-cloud", "planning_temperature":0.3, "execution_temperature":0.2},
    {"id":"2", "model":"gemma4:31b-cloud", "planning_temperature":0.3, "execution_temperature":0.2},
    {"id":"Alpha", "model":"gemma4:31b-cloud", "planning_temperature":0.3, "execution_temperature":0.2}
  ],
  "sources": ["server-rs/src/agent/runner/mission_tools/lifecycle.rs"],
  "rubric": {"root_cause":4, "trigger":2, "correction_and_regression":2, "evidence":1, "handoff":1},
  "history_policy": "fresh history; do not include prior findings or the hidden regression"
}
```

```powershell
python execution/mission_benchmark.py freeze --case case.json --out frozen.json
```

The resulting fingerprint includes the brief, model settings, history policy, rubric, and source hashes. Verify actual runtime settings and source snapshots against this record before dispatch. A declared history policy does not reset agent memory automatically. A fresh blind trial requires actual history isolation. Keep any hidden diagnosis and reproduction outside the agents' authorized workspace. Cloud source-sharing and spending require the applicable operator authorization; this recorder grants neither.

## Record execution and independent review

A run file identifies its `case_fingerprint` and supplies independently checked `execution_checks`: `handoffs_once`, `reports_delivered`, `scope_preserved`, and `budget_preserved`, each a boolean. Derive these from receipts, tool execution records, and budget snapshots, rather than agent assertions.

Without a grade file, scoring returns `UNSCORED`, including when administrative mission status is `completed`:

```powershell
python execution/mission_benchmark.py score --frozen frozen.json --run run.json --out assessment.json
```

A grade file identifies the same fingerprint, declares `reviewer_type: "independent"`, lists local `evidence_files`, and supplies `points` for every rubric criterion. Evidence files must exist and are hashed into the assessment. The independent reviewer must actually reproduce or reject the finding; the recorder cannot infer correctness from prose or authenticate a reviewer merely from that label.

```powershell
python execution/mission_benchmark.py score --frozen frozen.json --run run.json --grade independent-grade.json --out assessment.json
python execution/mission_benchmark.py compare first-assessment.json second-assessment.json --out comparison.json
```

Quality passes only when all required execution checks pass and the independently awarded score reaches the rubric maximum. Different fingerprints are not comparable as repeat trials. A controlled temperature experiment should group runs by the same task/source/history baseline while explicitly treating temperature as the changed variable; do not report those runs as identical-condition repeats.

## Runtime completion policy

- A safe read-only run can finish with `verification_status: "not_required"`; that does not certify its factual assertions.
- When verification is required, recognized test execution must finish successfully after the last mutation. Printed test names and quoted passing output do not establish a witness. A later failed test supersedes an earlier pass.
- Missing evidence preserves the report as `MISSION_REVIEW_REQUIRED` and pauses the owned mission ledger. Shared child ledgers remain owned by the parent.
- Rejected, failed, empty, or unverified completion cannot become verified success. A completion call runs alone so it cannot race another tool in the same batch.
- Stable `assignment_id` values identify repeated delegated work. Completed results are reused. Failed or interrupted assignments require a deliberate review/retry decision; a new assignment uses a new ID. Results are isolated by mission, parent, capability scope, and tool arguments.
- Cost events and receipts describe engine estimates. `pricing_source` identifies configured rates, static registry rates, local execution, or fallback estimates. Partial custom pricing retains an estimate for the missing token direction. Provider invoices remain the source for actual charges.

Process witnesses establish execution evidence, not the completeness or quality of a test suite. Business transactions need their own validated action receipts and acceptance scenarios; a maintenance test run does not establish SMB workflow success.

Supervisors retain worker reports, but a worker's process records are not automatically promoted into the parent's verification record. Root verification therefore needs an independent fresh witness. Likewise, replaying a persisted assignment result does not certify the current source revision after a restart or later edits.

## Local regression checks

```powershell
cargo test --manifest-path server-rs/Cargo.toml --config profile.test.opt-level=0 --bin server-rs agent::runner::
python -m unittest discover -s tests -p test_mission_benchmark.py
```

The SQL migration adds a dedicated delegation ledger and cleanup triggers. Rolling back the binary can safely leave that additive table in place; do not remove live assignment records to force a retry. Results are deleted when their mission or source agent is deleted.
