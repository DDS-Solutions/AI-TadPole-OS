#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / skillify / skill_scaffolder
- **Primary Entrypoints**: `main`

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
from typing import Dict, List, Any

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

SKILL_TEMPLATE = """---
name: {name}
description: "{description}"
when_to_use: "{when_to_use}"
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: {priority}
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / {name}
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# {title}

> **Purpose**: {description}  
> **Workflow Bindings**: Integrated into [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md) and [`/enhance`](file:///D:/TadpoleOS-Dev/.agent/workflows/enhance.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/{name}_patterns.md`](./references/{name}_patterns.md) | Deep schemas, comprehensive matrices, and protocol patterns | Deep design & edge-case handling |
| [`scripts/{script_name}.py`](./scripts/{script_name}.py) | Deterministic automation script with built-in self-tests | Executing and verifying skill actions |

---

## 🧭 1. Procedural Execution Guidelines

1. **Step 1**: Analyze requirements and verify input parameters.
2. **Step 2**: Execute core deterministic operations.
3. **Step 3**: Validate outcomes against verification gates.

---

## 🧠 2. Socratic Gate for {title}

Before executing this workflow, verify:
- **Gate 1 (Input Validity)**: Are all inputs and target files verified to exist?
- **Gate 2 (Side Effect Safety)**: Are operations confined to their declared scope?
- **Gate 3 (Deterministic Proof)**: Is there a test or verification command proving success?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Ad-Hoc Scripts** | Writing unversioned scratch scripts with no tests. | Commit deterministic Python scripts in `scripts/` with `--test`. |
| **Loose Documentation** | Dumping dozens of loose `.md` files in the skill root. | Move all reference documents into `references/`. |
| **Unbounded Scope** | Attempting to handle unrelated domains in a single skill. | Decompose into distinct specialized skills. |

---

## 🛠️ 4. Deterministic Verification Gate

Validate execution outcomes:

```powershell
# 1. Run built-in self-tests
python .agent/skills/{name}/scripts/{script_name}.py --test
```
"""

SCRIPT_TEMPLATE = """#!/usr/bin/env python3
\"\"\"
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / {name} / {script_name}
- **Primary Entrypoints**: `execute_action`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[{script_name}]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
\"\"\"

import sys
import os
import argparse
from pathlib import Path

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

def execute_action(target: str) -> bool:
    \"\"\"Execute the primary deterministic action for {name}.\"\"\"
    return True

def run_self_tests() -> bool:
    \"\"\"Run built-in self-tests.\"\"\"
    assert execute_action("test"), "Self-test action execution failed"
    print("[PASS] {script_name} self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="{name} automation tool")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
"""

def scaffold_skill(name: str, target_dir: Path, description: str, when_to_use: str, priority: str = "HIGH") -> Path:
    """Scaffold a new Sovereign skill adhering to L1/L2/L3 specification."""
    skill_dir = target_dir / name
    skill_dir.mkdir(parents=True, exist_ok=True)
    ref_dir = skill_dir / "references"
    ref_dir.mkdir(parents=True, exist_ok=True)
    scripts_dir = skill_dir / "scripts"
    scripts_dir.mkdir(parents=True, exist_ok=True)

    script_name = name.replace("-", "_") + "_tool"
    title = name.replace("-", " ").title()

    skill_file = skill_dir / "SKILL.md"
    skill_content = SKILL_TEMPLATE.format(
        name=name,
        title=title,
        description=description,
        when_to_use=when_to_use,
        priority=priority,
        script_name=script_name
    )
    skill_file.write_text(skill_content, encoding="utf-8")

    # Scaffold default reference doc
    ref_file = ref_dir / f"{name}_patterns.md"
    ref_content = f"# {title} Reference Patterns\n\nDeep architectural specifications and patterns for {name}.\n"
    ref_file.write_text(ref_content, encoding="utf-8")

    # Scaffold default script
    script_file = scripts_dir / f"{script_name}.py"
    script_content = SCRIPT_TEMPLATE.format(
        name=name,
        script_name=script_name
    )
    script_file.write_text(script_content, encoding="utf-8")

    return skill_dir

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        skill_dir = scaffold_skill(
            name="demo-skill",
            target_dir=tmp_path,
            description="A test demo skill for validation.",
            when_to_use="When validating skill scaffolding."
        )
        assert (skill_dir / "SKILL.md").exists(), "SKILL.md missing"
        assert (skill_dir / "references" / "demo-skill_patterns.md").exists(), "Reference doc missing"
        assert (skill_dir / "scripts" / "demo_skill_tool.py").exists(), "Script tool missing"

    print("[PASS] skill_scaffolder self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Sovereign Skill Scaffolder (L1/L2/L3 Standard)")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--name", type=str, help="Kebab-case skill name")
    parser.add_argument("--desc", type=str, help="Description for skill")
    parser.add_argument("--when", type=str, help="When to use trigger statement")
    parser.add_argument("--priority", type=str, default="HIGH", help="CRITICAL, HIGH, or MEDIUM")
    parser.add_argument("--target-dir", type=str, default=".agent/skills", help="Target skills directory")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.name:
        if not args.desc or not args.when:
            print("[ERROR] --desc and --when are required when creating a skill")
            sys.exit(1)
        target = Path(args.target_dir)
        skill_path = scaffold_skill(args.name, target, args.desc, args.when, args.priority)
        print(f"[PASS] Successfully scaffolded Sovereign skill at: {skill_path}")
        sys.exit(0)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
