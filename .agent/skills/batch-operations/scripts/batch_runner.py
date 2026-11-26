#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / batch-operations / batch_runner
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import re
import argparse
from pathlib import Path

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')


def process_batch(root_dir: Path, glob_pattern: str, find_str: str, replace_str: str, dry_run: bool = True) -> dict:
    matched_files = []
    total_replacements = 0

    for file_path in root_dir.glob(glob_pattern):
        if not file_path.is_file():
            continue

        try:
            content = file_path.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue

        count = len(re.findall(re.escape(find_str), content))
        if count > 0:
            matched_files.append({"file": str(file_path), "matches": count})
            total_replacements += count
            if not dry_run:
                new_content = content.replace(find_str, replace_str)
                file_path.write_text(new_content, encoding="utf-8")

    return {
        "files_matched": len(matched_files),
        "total_occurrences": total_replacements,
        "dry_run": dry_run,
        "details": matched_files
    }


def run_self_tests():
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        f1 = td_path / "a.txt"
        f2 = td_path / "b.txt"
        f1.write_text("Hello OLD_VAL World OLD_VAL", encoding="utf-8")
        f2.write_text("Unrelated", encoding="utf-8")

        # 1. Test Dry Run
        res = process_batch(td_path, "*.txt", "OLD_VAL", "NEW_VAL", dry_run=True)
        assert res["files_matched"] == 1
        assert res["total_occurrences"] == 2
        assert "OLD_VAL" in f1.read_text(encoding="utf-8")

        # 2. Test Execution
        res2 = process_batch(td_path, "*.txt", "OLD_VAL", "NEW_VAL", dry_run=False)
        assert res2["total_occurrences"] == 2
        assert "NEW_VAL" in f1.read_text(encoding="utf-8")
        assert "OLD_VAL" not in f1.read_text(encoding="utf-8")

        print("[PASS] batch_runner self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Batch Search and Replace Runner")
    parser.add_argument("--find", help="Target string to find")
    parser.add_argument("--replace", help="Replacement string")
    parser.add_argument("--glob", default="**/*", help="File glob pattern")
    parser.add_argument("--dir", default=".", help="Root directory")
    parser.add_argument("--execute", action="store_true", help="Apply changes (default is dry-run)")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    if not args.find or args.replace is None:
        parser.print_help()
        sys.exit(1)

    res = process_batch(Path(args.dir), args.glob, args.find, args.replace, dry_run=not args.execute)
    mode = "DRY RUN PREVIEW" if res["dry_run"] else "EXECUTED CHANGES"
    print(f"[{mode}] Found {res['total_occurrences']} occurrences across {res['files_matched']} files.")
    for d in res["details"]:
        print(f"  - {d['file']}: {d['matches']} matches")


if __name__ == "__main__":
    main()
