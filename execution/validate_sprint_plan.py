#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / validate_sprint_plan
- **Primary Entrypoints**: `validate_text`, `main`

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

# Ensure UTF-8 output on Windows consoles
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent

# Canonical Enums per docs/wiki/Glossary.md
CANONICAL_MISSION_STATUS = {"Pending", "SpecReview", "Active", "Completed", "Failed", "Paused"}
CANONICAL_SUBSYSTEM_STATUS = {"NotStarted", "Warming", "Ready", "Failed"}
CANONICAL_SYSTEM_HEALTH = {"Warming", "Ready", "Degraded"}

# Known hallucinated routes that do not exist in Tadpole OS
FORBIDDEN_ROUTES = ["/api/v3", "/api/v2", "/v2/"]

# Known hallucinated state indicators claimed as canonical
FABRICATED_CANONICAL_STATES = ["isProcessing", "isQueuing", "isFailed"]


def get_registered_skills():
    skills_dir = WORKSPACE_ROOT / ".agent" / "skills"
    if not skills_dir.exists():
        return set()
    return {d.name.lower() for d in skills_dir.iterdir() if d.is_dir()}


def validate_text(text: str):
    errors = []
    warnings = []
    grounded_checks = []

    # 1. Check for fabricated API routes
    for route in FORBIDDEN_ROUTES:
        if route in text:
            errors.append(
                f"[ROUTE-HALLUCINATION] Referenced fictitious API route '{route}'. "
                f"Tadpole OS routes are strictly Axum /v1 (server-rs/src/router.rs)."
            )

    # 2. Check for cited report / documentation files that do not exist
    report_matches = re.findall(r"(?:`([a-zA-Z0-9_\-/\\]+\.md)`|\b([a-zA-Z0-9_\-]+(?:_Report|_report|_audit|_drift)\.md)\b)", text)
    cited_files = set()
    for m in report_matches:
        f = m[0] or m[1]
        if f:
            cited_files.add(f)

    for file_name in cited_files:
        candidate_paths = [
            WORKSPACE_ROOT / file_name,
            WORKSPACE_ROOT / "docs" / file_name,
            WORKSPACE_ROOT / "reports" / file_name,
            WORKSPACE_ROOT / "directives" / file_name,
            WORKSPACE_ROOT / "docs" / "wiki" / file_name,
        ]
        exists = any(p.exists() for p in candidate_paths)
        if not exists:
            # If the text explicitly claims it is documented in this file or audit is complete
            if any(k in text for k in ["documented in", "Discovery Phase", "Audit Complete", "findings are", "verified against"]):
                errors.append(
                    f"[MISSING-ARTIFACT] Claimed completion or documented findings in '{file_name}', but the file does not exist on disk."
                )
            else:
                warnings.append(
                    f"[UNRESOLVED-FILE] Referenced file '{file_name}' does not exist on disk."
                )
        else:
            grounded_checks.append(f"Artifact verified on disk: {file_name}")

    # 3. Check for fabricated UI / Engine State definitions
    for state in FABRICATED_CANONICAL_STATES:
        if state in text and any(kw in text.lower() for kw in ["canonical state", "state definition", "state model"]):
            errors.append(
                f"[STATE-MODEL-DRIFT] '{state}' is an ad-hoc UI boolean, not a canonical engine state enum. "
                f"Canonical enums in docs/wiki/Glossary.md are: MissionStatus {CANONICAL_MISSION_STATUS}, "
                f"SubsystemStatus {CANONICAL_SUBSYSTEM_STATUS}."
            )

    # 4. Check specialist roles against .agent/skills
    registered_skills = get_registered_skills()
    if registered_skills:
        # Match words following "specialists:" or "specialist"
        specialist_blocks = re.findall(r"(?:Specialists?|specialists?):\s*([^\n\*\.]+)", text)
        for block in specialist_blocks:
            # extract potential agent tokens
            tokens = re.findall(r"`?([a-zA-Z0-9_\-]+)`?", block)
            for token in tokens:
                token_lower = token.lower()
                # Ignore generic role descriptors
                if token_lower in ["coder", "auditor", "specialist", "and", "or", "lead", "engineer"]:
                    continue
                if token_lower not in registered_skills and f"{token_lower}-expert" not in registered_skills:
                    warnings.append(
                        f"[ROLE-DRIFT] Specialist '{token}' is not registered in .agent/skills/."
                    )

    # 5. Legacy Terminology Advisory (Alpha Node vs Department Lead)
    if "Alpha Node" in text or "alpha node" in text:
        warnings.append(
            "[TERMINOLOGY-ADVISORY] 'Alpha Node' is a deprecated term per docs/wiki/Glossary.md. "
            "Consider updating to canonical 'Department Lead' (internal ID: 'alpha')."
        )

    return errors, warnings, grounded_checks


def main():
    parser = argparse.ArgumentParser(description="Deterministic Sprint Plan & Drift Report Validator")
    parser.add_argument("--file", "-f", type=str, help="Path to markdown plan or report file")
    parser.add_argument("--text", "-t", type=str, help="Raw text string to validate")

    args = parser.parse_args()

    content = ""
    if args.file:
        file_path = Path(args.file)
        if not file_path.is_absolute():
            file_path = WORKSPACE_ROOT / file_path
        if not file_path.exists():
            print(f"Error: Target file '{file_path}' does not exist.")
            sys.exit(1)
        content = file_path.read_text(encoding="utf-8")
    elif args.text:
        content = args.text
    elif not sys.stdin.isatty():
        content = sys.stdin.read()
    else:
        parser.print_help()
        sys.exit(1)

    errors, warnings, grounded_checks = validate_text(content)

    print("=" * 60)
    print(" 🐸 Tadpole OS Deterministic Sprint Grounding Validator")
    print("=" * 60)

    for check in grounded_checks:
        print(f"  [OK] {check}")

    for warn in warnings:
        print(f"  [WARN] {warn}")

    if errors:
        print("\n❌ VALIDATION FAILED: Hallucinated claims or architectural drift detected:")
        for err in errors:
            print(f"  - {err}")
        print("\nAction: Refactor the proposal to align with verified ground truth.")
        sys.exit(1)
    else:
        print("\n✅ VALIDATION PASSED: Plan is grounded in codebase reality.")
        sys.exit(0)


if __name__ == "__main__":
    main()
