#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / handoff / handoff_manager
- **Primary Entrypoints**: `validate_handoff_document`, `scaffold_handoff_document`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic handoff validation and scaffolding without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[handoff_manager]`, `[VALIDATION]`, `[SCAFFOLD]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import re
import json
import argparse
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

MANDATORY_SECTIONS: List[Tuple[str, str]] = [
    (r'##\s*1\.\s*(?:goal\s*(?:&|and)\s*context|primary\s*objective)', "Goal & Context"),
    (r'##\s*2\.\s*(?:completed\s*(?:milestones|work)|architectural\s*decisions)', "Completed Milestones & Decisions"),
    (r'##\s*3\.\s*(?:modified\s*files(?:\s*manifest)?|files\s*touched)', "Modified Files Manifest"),
    (r'##\s*4\.\s*(?:verification\s*commands|test\s*results)', "Verification Commands & Test Results"),
    (r'##\s*5\.\s*(?:active\s*blockers|known\s*issues|pending\s*work)', "Active Blockers & Known Issues"),
    (r'##\s*6\.\s*(?:cold-start\s*resume\s*protocol|next\s*(?:actions|steps|skills))', "Cold-Start Resume Protocol"),
]

PLACEHOLDER_PATTERN = re.compile(r'<[A-Za-z0-9_\s\-\:/\.]+>|\bTODO\b|\bFIXME\b')

def validate_handoff_document(content: str, file_path: Optional[Path] = None, check_file_existence: bool = True) -> Dict[str, Any]:
    """
    Validate that a handoff markdown document adheres to the Sovereign 6-section schema,
    contains no unresolved template placeholders, and references valid code paths.
    """
    errors: List[str] = []
    warnings: List[str] = []
    content_lower = content.lower()

    # 1. Mandatory Section Validation
    present_sections: List[str] = []
    missing_sections: List[str] = []
    for pattern, name in MANDATORY_SECTIONS:
        if re.search(pattern, content_lower):
            present_sections.append(name)
        else:
            missing_sections.append(name)
            errors.append(f"Missing mandatory section: '{name}'")

    # 2. Template Placeholder Check
    placeholders = PLACEHOLDER_PATTERN.findall(content)
    # Ignore acceptable inline code block samples like <timestamp> or <Task Title> if explicitly inside schema documentation
    filtered_placeholders = [p for p in placeholders if not p.startswith("<line_number>") and not p.startswith("<N>")]
    if filtered_placeholders:
        sample = filtered_placeholders[:5]
        warnings.append(f"Unpopulated template placeholders or TODOs found ({len(filtered_placeholders)} total): {sample}")

    # 3. Executable Code / Commands Check
    has_code_block = "```" in content
    if not has_code_block:
        warnings.append("Handoff document contains no executable code blocks or verification command snippets.")

    # 4. Check Referenced File Paths (if file_path or workspace is available)
    referenced_files = re.findall(r'\[([^\]]+)\]\((file:///[^)]+|[a-zA-Z0-9_\-\./\\]+\.[a-zA-Z0-9]+)\)', content)
    missing_file_paths: List[str] = []
    if check_file_existence and referenced_files:
        workspace_root = Path.cwd()
        for label, url in referenced_files:
            # Strip file:/// prefix
            clean_path = url.replace("file:///", "").replace("file://", "")
            # Normalize path
            target_path = Path(clean_path)
            if not target_path.is_absolute():
                target_path = workspace_root / clean_path
            
            # Check existence if not pointing to hypothetical sample or .tmp
            if not target_path.exists() and not ".tmp" in str(target_path) and not "path/to/" in str(target_path):
                missing_file_paths.append(str(url))

        if missing_file_paths:
            warnings.append(f"Referenced {len(missing_file_paths)} non-existent or relative file path(s): {missing_file_paths[:3]}")

    is_valid = len(errors) == 0
    return {
        "valid": is_valid,
        "errors": errors,
        "warnings": warnings,
        "present_sections": present_sections,
        "missing_sections": missing_sections,
        "placeholders_found": len(filtered_placeholders),
        "total_chars": len(content),
        "word_count": len(content.split())
    }

def scaffold_handoff_document(title: str, output_path: Optional[Path] = None, source_agent: str = "Tadpole Sovereign Agent", target_agent: str = "Incoming Agent / Next Session") -> str:
    """
    Generate a pristine, standardized session handoff document pre-populated with timestamp
    and the 6 mandatory schema sections.
    """
    now_iso = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    template = f"""# Session Handoff: {title}

> **Handoff Timestamp**: {now_iso}
> **Source Agent**: {source_agent}
> **Target Agent**: {target_agent}
> **Working Directory**: `D:/TadpoleOS-Dev`

---

## 1. Goal & Context
- **Primary Objective**: {title}
- **Current Lifecycle State**: IMPLEMENT
- **Session Background**: Context compacted to transfer execution without token overhead.

## 2. Completed Milestones & Architectural Decisions
- **Milestones Completed**:
  - Initialized task state and confirmed preconditions.
- **Architectural Invariants & Decisions**:
  - All modifications must comply with Sovereign Active Documentation Guard (ADG v2).
  - All executable tools must maintain deterministic self-tests.

## 3. Modified Files Manifest
| File Path | Change Description | Symbols Touched |
|---|---|---|
| [`SKILL.md`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/SKILL.md) | Standardized metadata and protocol bindings | `handoff` |

## 4. Verification Commands & Test Results
- **Executable Verification Commands**:
  ```powershell
  python .agent/skills/handoff/scripts/handoff_manager.py --test
  npm run adg:verify
  ```
- **Latest Test Status**:
  - `[PASS]` `handoff_manager.py --test` (All unit tests verified)

## 5. Active Blockers & Known Issues
None. All active gates green.

## 6. Cold-Start Resume Protocol
1. **Next Immediate Action**: Review modified manifest and run declared verification commands.
2. **Recommended Skills**:
   - [`handoff`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/SKILL.md)
3. **Execution Command**:
   ```powershell
   python .agent/skills/handoff/scripts/handoff_manager.py --test
   ```
"""
    if output_path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(template, encoding='utf-8')
    
    return template

def run_self_tests() -> bool:
    """
    Execute comprehensive in-memory unit tests for handoff validation and scaffolding.
    """
    print("[handoff_manager] Executing self-test suite...")

    # Test 1: Valid generated scaffold passes validation
    scaffold = scaffold_handoff_document("Self Test Verification")
    res1 = validate_handoff_document(scaffold, check_file_existence=False)
    if not res1["valid"]:
        print(f"[handoff_manager] [FAIL] Self-test 1 (valid scaffold) failed: {res1['errors']}")
        return False

    # Test 2: Missing mandatory section produces error
    broken_content = """# Session Handoff: Broken
## 1. Goal & Context
Some goal.
## 2. Completed Milestones & Architectural Decisions
Done.
## 3. Modified Files Manifest
None.
## 4. Verification Commands & Test Results
None.
## 5. Active Blockers & Known Issues
None.
"""
    res2 = validate_handoff_document(broken_content, check_file_existence=False)
    if res2["valid"] or "Cold-Start Resume Protocol" not in res2["missing_sections"]:
        print(f"[handoff_manager] [FAIL] Self-test 2 (missing section) did not detect missing Cold-Start Protocol")
        return False

    # Test 3: Unpopulated template placeholders detection
    placeholder_content = scaffold + "\n- <Unresolved Parameter>\n"
    res3 = validate_handoff_document(placeholder_content, check_file_existence=False)
    if res3["placeholders_found"] == 0:
        print("[handoff_manager] [FAIL] Self-test 3 did not detect unpopulated placeholder")
        return False

    # Test 4: JSON serialization sanity
    try:
        json_dump = json.dumps(res1)
        assert len(json_dump) > 0
    except Exception as e:
        print(f"[handoff_manager] [FAIL] Self-test 4 JSON serialization failed: {e}")
        return False

    print("[handoff_manager] [VALIDATION] Schema rules verified.")
    print("[handoff_manager] [SCAFFOLD] Template generation verified.")
    print("[handoff_manager] [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Deterministic Sovereign Session Handoff Manager & Validator."
    )
    parser.add_argument("file", nargs="?", type=str, help="Path to handoff markdown file to validate.")
    parser.add_argument("--validate", type=str, help="Explicit path to handoff markdown file to validate.")
    parser.add_argument("--scaffold", action="store_true", help="Generate a new standardized handoff document.")
    parser.add_argument("--title", type=str, default="Task Execution Session", help="Title for scaffolded handoff.")
    parser.add_argument("--output", type=str, help="Output path for scaffolded handoff document.")
    parser.add_argument("--json", action="store_true", help="Output results in structured JSON format.")
    parser.add_argument("--test", action="store_true", help="Run automated self-tests.")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.scaffold:
        now_str = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S")
        out_path = Path(args.output) if args.output else Path(".tmp") / "handoffs" / f"handoff-{now_str}.md"
        scaffold = scaffold_handoff_document(args.title, output_path=out_path)
        if args.json:
            print(json.dumps({
                "status": "success",
                "output_path": str(out_path),
                "title": args.title,
                "timestamp": datetime.now(timezone.utc).isoformat()
            }, indent=2))
        else:
            print(f"[handoff_manager] [SCAFFOLD] Successfully created handoff artifact: {out_path}")
        sys.exit(0)

    target_file = args.validate or args.file
    if target_file:
        path = Path(target_file)
        if not path.exists():
            if args.json:
                print(json.dumps({"valid": False, "error": f"File not found: {target_file}"}, indent=2))
            else:
                print(f"[handoff_manager] [FAIL] File not found: {target_file}")
            sys.exit(1)

        content = path.read_text(encoding='utf-8', errors='replace')
        res = validate_handoff_document(content, file_path=path)

        if args.json:
            print(json.dumps(res, indent=2))
        else:
            print(f"[handoff_manager] [VALIDATION] Auditing handoff file: {target_file}")
            print(f"Status: {'VALID' if res['valid'] else 'INVALID'}")
            print(f"Sections present: {len(res['present_sections'])}/6")
            if res['errors']:
                print("Errors:")
                for err in res['errors']:
                    print(f"  ❌ {err}")
            if res['warnings']:
                print("Warnings:")
                for w in res['warnings']:
                    print(f"  ⚠️ {w}")
            if res['valid']:
                print("[handoff_manager] [PASS] Handoff document conforms to Sovereign schema.")
            else:
                print("[handoff_manager] [FAIL] Handoff document failed validation.")

        sys.exit(0 if res['valid'] else 1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
