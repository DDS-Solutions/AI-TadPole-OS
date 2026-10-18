#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / memory-system / memory_manager
- **Primary Entrypoints**: `audit_memory_integrity`, `scan_memory_for_secrets`, `record_memory_entry`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic memory index auditing and secret scanning without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[memory_manager]`, `[AUDIT]`, `[SECRETS]`, `[RECORD]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import re
import json
import argparse
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Any, Optional, Tuple

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

POINTER_PATTERN = re.compile(r'-\s*\[([a-zA-Z0-9_\-]+)\]\s*(.*?)\s*→\s*([a-zA-Z0-9_\-\./\\]+\.md)', re.IGNORECASE)

SECRET_PATTERNS = [
    (re.compile(r'\bsk-[a-zA-Z0-9]{20,}\b'), "OpenAI API Key"),
    (re.compile(r'\bgh[pousr]_[a-zA-Z0-9]{20,}\b'), "GitHub Token"),
    (re.compile(r'\bAIza[0-9A-Za-z-_]{35}\b'), "Google API Key"),
    (re.compile(r'-----BEGIN\s+(?:RSA|OPENSSH|EC)?\s*PRIVATE\s+KEY-----'), "Private Key Header"),
    (re.compile(r'(?:api[_-]?key|password|bearer|secret)\s*[:=]\s*["\']([a-zA-Z0-9_\-\.]{12,})["\']', re.IGNORECASE), "Generic Hardcoded Secret")
]

def scan_memory_for_secrets(memory_dir: Path) -> List[Dict[str, Any]]:
    """Scan all markdown files in memory directory for credential leaks."""
    findings: List[Dict[str, Any]] = []
    if not memory_dir.exists():
        return findings

    for md_file in memory_dir.glob("*.md"):
        try:
            content = md_file.read_text(encoding='utf-8', errors='ignore')
            for line_idx, line in enumerate(content.splitlines(), start=1):
                # Ignore schema examples in documentation
                if "<" in line and ">" in line:
                    continue
                for pattern, secret_type in SECRET_PATTERNS:
                    if pattern.search(line):
                        findings.append({
                            "file": md_file.name,
                            "line": line_idx,
                            "type": secret_type,
                            "preview": line.strip()[:60] + "..."
                        })
        except Exception:
            continue

    return findings

def audit_memory_integrity(memory_dir: Path) -> Dict[str, Any]:
    """Audit MEMORY.md pointer index, check topic file existence, and measure line bloat."""
    errors: List[str] = []
    warnings: List[str] = []

    memory_index = memory_dir / "MEMORY.md"
    if not memory_index.exists():
        errors.append(f"Root memory index missing: {memory_index}")
        return {"valid": False, "errors": errors, "warnings": warnings, "line_count": 0, "pointers": []}

    content = memory_index.read_text(encoding='utf-8', errors='ignore')
    lines = content.splitlines()
    line_count = len(lines)

    if line_count > 200:
        warnings.append(f"MEMORY.md line count ({line_count}) exceeds the 200-line ceiling. Compaction advised.")

    # Parse pointers
    pointers: List[Dict[str, str]] = []
    referenced_topics: set = set()
    for match in POINTER_PATTERN.finditer(content):
        cat, summary, target_file = match.groups()
        target_name = Path(target_file).name
        pointers.append({
            "category": cat.strip(),
            "summary": summary.strip(),
            "target": target_name
        })
        referenced_topics.add(target_name)

    # Check if pointed topic files exist
    broken_pointers: List[str] = []
    for p in pointers:
        target_path = memory_dir / p["target"]
        if not target_path.exists():
            broken_pointers.append(f"Pointer points to missing topic file: '{p['target']}' (Summary: {p['summary'][:40]})")
            errors.append(broken_pointers[-1])

    # Check for orphaned topic files
    all_topic_files = {f.name for f in memory_dir.glob("*.md") if f.name != "MEMORY.md"}
    orphans = all_topic_files - referenced_topics
    for o in orphans:
        warnings.append(f"Orphaned topic file not indexed in MEMORY.md: '{o}'")

    is_valid = len(errors) == 0
    return {
        "valid": is_valid,
        "line_count": line_count,
        "pointers_count": len(pointers),
        "pointers": pointers,
        "broken_pointers": broken_pointers,
        "orphaned_files": list(orphans),
        "errors": errors,
        "warnings": warnings
    }

def record_memory_entry(memory_dir: Path, category: str, summary: str, topic_file: str, detail: Optional[str] = None) -> bool:
    """Deterministically append a pointer to MEMORY.md and update the corresponding topic file."""
    memory_dir.mkdir(parents=True, exist_ok=True)
    memory_index = memory_dir / "MEMORY.md"

    # Normalize topic filename
    topic_name = topic_file if topic_file.endswith(".md") else f"{topic_file}.md"
    topic_path = memory_dir / topic_name

    # 1. Update topic file
    now_date = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    if not topic_path.exists():
        header = f"""---
type: {category}
created: {now_date}
updated: {now_date}
---

# {topic_name.replace('.md', '').replace('-', ' ').title()}
"""
        topic_path.write_text(header, encoding='utf-8')

    if detail:
        with open(topic_path, "a", encoding="utf-8") as tf:
            tf.write(f"\n## Entry ({now_date})\n- {detail}\n")

    # 2. Append pointer to MEMORY.md
    if not memory_index.exists():
        base_index = """# Memory Index

## Active Memory Pointers
"""
        memory_index.write_text(base_index, encoding='utf-8')

    pointer_line = f"- [{category}] {summary} → {topic_name}\n"
    with open(memory_index, "a", encoding="utf-8") as mf:
        mf.write(pointer_line)

    return True

def run_self_tests() -> bool:
    """Execute comprehensive automated self-tests."""
    print("[memory_manager] Executing self-test suite...")

    test_dir = Path(".tmp/test_memory_system")
    test_dir.mkdir(parents=True, exist_ok=True)

    try:
        # Test 1: Record and audit valid memory
        record_memory_entry(test_dir, "project", "Use relative paths everywhere", "conventions.md", "Enforce relative links.")
        audit_res = audit_memory_integrity(test_dir)
        assert audit_res["valid"], f"Audit failed on fresh entry: {audit_res['errors']}"
        assert audit_res["pointers_count"] == 1
        print("[memory_manager] [RECORD] Memory recording & pointer appending verified.")
        print("[memory_manager] [AUDIT] Memory pointer resolution verified.")

        # Test 2: Broken pointer detection
        mem_index = test_dir / "MEMORY.md"
        with open(mem_index, "a", encoding="utf-8") as f:
            f.write("- [tech] Missing item → non_existent_topic.md\n")

        broken_res = audit_memory_integrity(test_dir)
        assert not broken_res["valid"]
        assert len(broken_res["broken_pointers"]) == 1
        print("[memory_manager] [AUDIT] Broken pointer detection verified.")

        # Test 3: Secret scanner detection
        secret_file = test_dir / "leaky.md"
        secret_file.write_text("API token: sk-abcdef1234567890abcdef1234567890", encoding='utf-8')
        secret_findings = scan_memory_for_secrets(test_dir)
        assert len(secret_findings) >= 1
        print("[memory_manager] [SECRETS] Credential leak scanner verified.")

    except Exception as e:
        print(f"[memory_manager] [FAIL] Self-test failed: {e}")
        return False
    finally:
        # Cleanup
        for f in test_dir.glob("*.md"):
            try:
                f.unlink()
            except Exception:
                pass
        try:
            test_dir.rmdir()
        except Exception:
            pass

    print("[memory_manager] [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Sovereign Cross-Session Memory Manager & Auditor."
    )
    parser.add_argument("path", nargs="?", default=".agent/memory", help="Path to memory directory (default: '.agent/memory').")
    parser.add_argument("--audit", action="store_true", help="Audit memory integrity and pointer health.")
    parser.add_argument("--scan-secrets", action="store_true", help="Scan memory files for credential leaks.")
    parser.add_argument("--add", action="store_true", help="Add a new memory entry.")
    parser.add_argument("--category", type=str, default="project", help="Category tag: user, project, tech, feedback.")
    parser.add_argument("--summary", type=str, help="Concise summary for MEMORY.md pointer.")
    parser.add_argument("--topic", type=str, default="project-conventions.md", help="Target topic filename.")
    parser.add_argument("--detail", type=str, help="Detailed context for the topic file.")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format.")
    parser.add_argument("--test", action="store_true", help="Run automated self-tests.")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    mem_dir = Path(args.path)

    if args.add:
        if not args.summary:
            print("[memory_manager] [FAIL] Error: --summary is required when adding a memory entry.")
            sys.exit(1)
        record_memory_entry(mem_dir, args.category, args.summary, args.topic, args.detail)
        print(f"[memory_manager] [RECORD] Added [{args.category}] '{args.summary}' -> {args.topic}")
        sys.exit(0)

    # Default to running audit and secret scan
    print("[memory_manager] Initializing persistent memory audit...")
    audit_res = audit_memory_integrity(mem_dir)
    secret_findings = scan_memory_for_secrets(mem_dir)

    is_healthy = audit_res["valid"] and len(secret_findings) == 0

    if args.json:
        report = {
            "directory": str(mem_dir),
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "healthy": is_healthy,
            "audit": audit_res,
            "secret_findings": secret_findings
        }
        print(json.dumps(report, indent=2))
    else:
        print("\n" + "=" * 60)
        print("  SOVEREIGN MEMORY SYSTEM AUDIT REPORT")
        print("=" * 60)
        print(f"Memory Directory: {mem_dir.resolve()}")
        print(f"Index Line Count: {audit_res['line_count']}/200 lines")
        print(f"Active Pointers : {audit_res['pointers_count']}")

        if secret_findings:
            print(f"\n[memory_manager] [FAIL] 🚨 DETECTED {len(secret_findings)} CREDENTIAL LEAKS:")
            for s in secret_findings:
                print(f"  ❌ {s['file']}:{s['line']} - {s['type']}")
        else:
            print("[memory_manager] [SECRETS] Zero credentials or secret leaks detected.")

        if audit_res["errors"]:
            print("\nErrors:")
            for err in audit_res["errors"]:
                print(f"  ❌ {err}")

        if audit_res["warnings"]:
            print("\nWarnings:")
            for warn in audit_res["warnings"]:
                print(f"  ⚠️ {warn}")

        print("\n" + "=" * 60)
        if is_healthy:
            print("[memory_manager] [PASS] Memory system is 100% healthy and aligned.")
            sys.exit(0)
        else:
            print("[memory_manager] [FAIL] Memory system audit identified violations.")
            sys.exit(1)

if __name__ == "__main__":
    main()
