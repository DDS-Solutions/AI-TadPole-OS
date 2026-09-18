#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / deduplicate_metadata
- **Primary Entrypoints**: `process_file`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[WRITTEN]`, `[ERROR]`
- **Witness Tests**: none declared
"""

import os
import re
import sys
from pathlib import Path

# Regex to match:
# // Metadata: [tag]
# # Metadata: [tag]
# [//]: # (Metadata: [tag])
# [//]: # "Metadata: [tag]"
METADATA_RE = re.compile(r'^\s*(?://|#|\[//\]:\s*#\s*[\("])\s*Metadata:\s*\[([^\]]+)\]\s*[\)"]?\s*$')

EXCLUDE_DIRS = {
    "node_modules",
    "target",
    "dist",
    ".git",
    "build",
    ".tmp",
    "coverage",
    ".code-review-graph",
    ".gemini"
}

ALLOWED_EXTENSIONS = {
    ".rs", ".ts", ".tsx", ".py", ".js", ".md", ".json", 
    ".toml", ".yaml", ".yml", ".css", ".html", ".sh", ".bat", ".ps1"
}

def process_file(file_path: Path, write: bool = False) -> bool:
    """Processes a file to detect and remove duplicate metadata comments.
    Returns True if duplicates were found and processed/written.
    """
    try:
        with open(file_path, 'r', encoding='utf-8', newline='') as f:
            content = f.read()
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return False

    # Detect newline style
    if '\r\n' in content:
        nl = '\r\n'
    else:
        nl = '\n'

    has_trailing_newline = content.endswith('\n')
    lines = content.splitlines()

    metadata_lines = []
    for idx, line in enumerate(lines):
        match = METADATA_RE.match(line)
        if match:
            tag = match.group(1)
            metadata_lines.append((idx, line, tag))

    if len(metadata_lines) <= 1:
        return False

    # Found duplicates!
    first_idx, first_line, first_tag = metadata_lines[0]
    duplicate_tags = [tag for _, _, tag in metadata_lines[1:]]
    print(f"[FOUND DUPS] {file_path.relative_to(Path('.'))}")
    print(f"  Keep line {first_idx + 1}: {first_line}")
    for idx, line, tag in metadata_lines[1:]:
        print(f"  Remove line {idx + 1}: {line}")

    if not write:
        return True

    # Keep only the first metadata line index, remove the rest
    to_remove = set(idx for idx, _, _ in metadata_lines[1:])
    new_lines = [line for idx, line in enumerate(lines) if idx not in to_remove]

    # Clean up trailing blank lines
    while new_lines and not new_lines[-1].strip():
        new_lines.pop()

    # Re-assemble content
    new_content = nl.join(new_lines)
    if has_trailing_newline or len(new_lines) > 0:
        new_content += nl

    try:
        with open(file_path, 'w', encoding='utf-8', newline='') as f:
            f.write(new_content)
        print(f"  [WRITTEN] Deduplicated {file_path.relative_to(Path('.'))}")
        return True
    except Exception as e:
        print(f"  [ERROR] Failed to write {file_path}: {e}", file=sys.stderr)
        return False

def main():
    write_mode = "--write" in sys.argv
    dry_run = not write_mode

    print("=== Tadpole OS Metadata Deduplicator ===")
    print(f"Mode: {'DRY RUN (Preview)' if dry_run else 'WRITE (Modify files)'}\n")

    root_dir = Path(".")
    dup_count = 0

    for root, dirs, files in os.walk(root_dir):
        # Exclude directories in-place
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

        for file in files:
            file_path = Path(root) / file
            if file_path.suffix.lower() not in ALLOWED_EXTENSIONS:
                continue

            if process_file(file_path, write=write_mode):
                dup_count += 1

    print(f"\nScan complete. Total files with duplicates found: {dup_count}")
    if dry_run and dup_count > 0:
        print("To apply these changes, run with: python execution/deduplicate_metadata.py --write")

if __name__ == "__main__":
    main()
