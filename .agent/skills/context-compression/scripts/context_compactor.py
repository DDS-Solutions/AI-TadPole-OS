#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / context-compression / context_compactor
- **Primary Entrypoints**: `analyze_context_footprint`, `verify_checkpoint_schema`, `generate_session_checkpoint`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic context estimation and schema verification without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[context_compactor]`, `[CONTEXT_AUDIT]`, `[CHECKPOINT]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import json
import re
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

REQUIRED_CHECKPOINT_SECTIONS = [
    (r'##\s*1\.\s*(?:user\s+)?objective', "User Objective / Goal"),
    (r'##\s*2\.\s*completed\s*(?:milestones|phases)', "Completed Milestones / Phases"),
    (r'##\s*3\.\s*(?:active\s+context|next\s+actions?)', "Active Context & Next Actions"),
    (r'##\s*4\.\s*architectural\s*decisions?', "Architectural Decisions & Invariants"),
    (r'##\s*5\.\s*modified\s*files', "Modified Files Manifest"),
]

def analyze_context_footprint(text: str) -> Dict[str, Any]:
    """Estimate token footprint and identify context degradation risk indicators."""
    lines = text.splitlines()
    total_lines = len(lines)
    words = len(text.split())
    estimated_tokens = int(words * 1.33)

    # Detect turns if transcript or message markers exist
    turn_matches = re.findall(r'(?:turn\s+\d+|user:|assistant:|step_\d+)', text, re.IGNORECASE)
    estimated_turns = len(turn_matches)

    risks = []
    if estimated_turns > 20:
        risks.append(f"Turn count ({estimated_turns}) exceeds 20-turn threshold. Level 3 compaction advised.")

    if estimated_tokens > 40000:
        risks.append(f"Estimated token footprint ({estimated_tokens}) is large. Context pruning recommended.")

    # Detect uncompressed raw dumps (>100 lines of repetitive output)
    if total_lines > 500 and "```" not in text:
        risks.append("Potential uncompressed raw log or file dump detected.")

    return {
        "total_lines": total_lines,
        "word_count": words,
        "estimated_tokens": estimated_tokens,
        "estimated_turns": estimated_turns,
        "compression_recommended": len(risks) > 0,
        "risk_indicators": risks
    }

def verify_checkpoint_schema(text: str) -> Dict[str, Any]:
    """Verify that a session checkpoint markdown document contains all mandatory sections."""
    text_lower = text.lower()
    missing_sections = []

    for pattern, name in REQUIRED_CHECKPOINT_SECTIONS:
        if not re.search(pattern, text_lower):
            missing_sections.append(name)

    is_compliant = len(missing_sections) == 0

    return {
        "compliant": is_compliant,
        "missing_sections": missing_sections,
        "total_checked": len(REQUIRED_CHECKPOINT_SECTIONS)
    }

def generate_session_checkpoint(
    objective: str,
    completed: List[str],
    next_steps: List[str],
    decisions: List[str],
    files: List[str]
) -> str:
    """Generate a standardized Level 3 Session Checkpoint markdown string."""
    completed_lines = "\n".join(f"- [x] {c}" for c in completed) if completed else "- [x] Initial task phase completed"
    next_lines = "\n".join(f"- [ ] {n}" for n in next_steps) if next_steps else "- [ ] Next milestone execution"
    decision_lines = "\n".join(f"{idx}. {d}" for idx, d in enumerate(decisions, 1)) if decisions else "1. Standard repository invariants maintained."
    file_lines = "\n".join(f"- `{f}`" for f in files) if files else "- No files recorded"

    return f"""# Session Checkpoint

## 1. User Objective
{objective}

## 2. Completed Milestones
{completed_lines}

## 3. Active Context & Next Actions
{next_lines}

## 4. Architectural Decisions & Invariants
{decision_lines}

## 5. Modified Files Manifest
{file_lines}
"""

def run_self_tests() -> bool:
    """Execute built-in self-tests verifying context analysis and checkpoint generation."""
    print("🔬 [context_compactor] Running self-test suite...")

    # Test 1: Checkpoint generation and validation
    doc = generate_session_checkpoint(
        objective="Harden skill registry files",
        completed=["Refactored clean-code", "Added unit tests"],
        next_steps=["Audit context-compression"],
        decisions=["Keep L2 body under 5k words", "Never handwrite v1 claims"],
        files=[".agent/skills/clean-code/SKILL.md"]
    )
    schema_res = verify_checkpoint_schema(doc)
    assert schema_res["compliant"], f"Test 1 Failed: Generated checkpoint failed schema: {schema_res}"

    # Test 2: Footprint analysis
    sample_text = "Turn 1\nHello\n" * 25
    footprint = analyze_context_footprint(sample_text)
    assert footprint["compression_recommended"], f"Test 2 Failed: Expected compression recommendation: {footprint}"

    # Test 3: Missing section detection
    incomplete_doc = "# Session Checkpoint\n## 1. User Objective\nTest\n"
    inc_res = verify_checkpoint_schema(incomplete_doc)
    assert not inc_res["compliant"] and len(inc_res["missing_sections"]) == 4, f"Test 3 Failed: {inc_res}"

    print("✅ [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic Context Compactor and Checkpoint Validator")
    parser.add_argument("--file", type=str, help="Path to transcript, session log, or checkpoint to evaluate")
    parser.add_argument("--verify-checkpoint", action="store_true", help="Verify checkpoint schema compliance")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    content = ""
    target_name = "stdin"

    if args.file:
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"❌ [FAIL] File not found: {file_path}", file=sys.stderr)
            sys.exit(1)
        target_name = str(file_path)
        content = file_path.read_text(encoding='utf-8', errors='replace')
    else:
        print("ℹ️  No input file provided. Running self-test suite.")
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print(f"🔍 [context_compactor] Auditing context footprint for: {target_name}")

    if args.verify_checkpoint or "checkpoint" in target_name.lower():
        res = verify_checkpoint_schema(content)
        print(f"📊 [CHECKPOINT] Compliant: {res['compliant']} | Missing Sections: {len(res['missing_sections'])}")

        if args.json:
            print(json.dumps(res, indent=2))
            sys.exit(0 if res["compliant"] else 1)

        if not res["compliant"]:
            print("❌ [FAIL] Checkpoint schema non-compliant. Missing mandatory sections:")
            for m in res["missing_sections"]:
                print(f"  ↳ {m}")
            sys.exit(1)
        else:
            print("✅ [PASS] Checkpoint conforms to Level 3 compaction standard.")
            sys.exit(0)
    else:
        audit = analyze_context_footprint(content)
        print(f"📡 [CONTEXT_AUDIT] Tokens: ~{audit['estimated_tokens']} | Turns: ~{audit['estimated_turns']} | Compaction Needed: {audit['compression_recommended']}")

        if args.json:
            print(json.dumps(audit, indent=2))
            sys.exit(0)

        if audit["compression_recommended"]:
            print("⚠️  [CONTEXT_AUDIT] Context degradation risks detected:")
            for r in audit["risk_indicators"]:
                print(f"  ↳ {r}")
            print("\nRecommended action: Generate Level 3 checkpoint via context_compactor.")
            sys.exit(0)
        else:
            print("✅ [PASS] Context footprint within acceptable limits.")
            sys.exit(0)

if __name__ == "__main__":
    main()
