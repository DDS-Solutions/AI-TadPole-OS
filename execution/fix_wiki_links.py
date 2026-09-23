#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / fix_wiki_links
- **Primary Entrypoints**: `fix_links`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import os
from pathlib import Path

WIKI_DIR = Path("docs/wiki")
REPO_BLOB_BASE = "https://github.com/DDS-Solutions/Tadpole-OS/blob/main"

REPLACEMENTS = [
    ("../GLOSSARY.md", f"{REPO_BLOB_BASE}/docs/GLOSSARY.md"),
    ("../ARCHITECTURE.md", f"{REPO_BLOB_BASE}/docs/ARCHITECTURE.md"),
    ("../CLI_TOOLS.md", f"{REPO_BLOB_BASE}/docs/CLI_TOOLS.md"),
    ("../../CONTRIBUTING.md", f"{REPO_BLOB_BASE}/CONTRIBUTING.md"),
    ("../../DEVELOPMENT.md", f"{REPO_BLOB_BASE}/DEVELOPMENT.md"),
    ("../../CODE_OF_CONDUCT.md", f"{REPO_BLOB_BASE}/CODE_OF_CONDUCT.md"),
    ("../../.env.example", f"{REPO_BLOB_BASE}/.env.example"),
    ("../../../server-rs/", f"{REPO_BLOB_BASE}/server-rs/"),
    ("../../../src/", f"{REPO_BLOB_BASE}/src/"),
    ("../../server-rs/", f"{REPO_BLOB_BASE}/server-rs/"),
    ("../../src/", f"{REPO_BLOB_BASE}/src/"),
]

def fix_links():
    count = 0
    for root, _, files in os.walk(WIKI_DIR):
        for f in files:
            if not f.endswith(".md"):
                continue
            fpath = Path(root) / f
            content = fpath.read_text(encoding="utf-8")
            new_content = content
            for old_str, new_str in REPLACEMENTS:
                new_content = new_content.replace(old_str, new_str)
            if new_content != content:
                fpath.write_text(new_content, encoding="utf-8")
                print(f"Fixed links in: {fpath}")
                count += 1
    print(f"Total wiki files updated: {count}")

if __name__ == "__main__":
    fix_links()
