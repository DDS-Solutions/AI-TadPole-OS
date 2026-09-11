#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / migrate_markdown_headers
- **Primary Entrypoints**: `derive_subsystem`, `extract_meta_fields`, `build_modern_header`, `migrate_markdown_file`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic header migration preserving YAML frontmatter, @docs references, and observability links.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[ERR]`, `[migrate_markdown_headers]`, `[MIGRATED]`
- **Witness Tests**: none declared
"""

import os
import re
import sys
import io
from pathlib import Path
from typing import Optional, Tuple

# Fix Windows console encoding for Unicode output
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

SKIP_DIRS = {
    '.git', 'node_modules', 'dist', 'target', 'build', '__pycache__',
    '.venv', 'venv', '.tmp', 'tmp', 'coverage', '3rdparty', 'scratch',
    'playwright-report', 'test-results', '.system_generated'
}

ROOT = Path(__file__).resolve().parent.parent

def derive_subsystem(file_path: Path) -> str:
    rel = file_path.relative_to(ROOT).as_posix()
    parts = rel.split('/')
    
    if rel in {'AGENTS.md', 'CLAUDE.md', 'GEMINI.md'}:
        return "Core Sovereign Governance / Entrypoints"
    elif rel in {'README.md', 'SECURITY.md', 'ROADMAP.md', 'SUPPORT.md'}:
        return f"Project Root / {file_path.stem}"
    elif rel.startswith('.agent/skills/'):
        skill_name = parts[2]
        return f"Agent Skills Registry / {skill_name}"
    elif rel.startswith('.agent/workflows/'):
        wf_name = parts[2].replace('.md', '')
        return f"Sovereign Workflows / {wf_name}"
    elif rel.startswith('.agent/agents/'):
        agent_name = parts[2].replace('.md', '')
        return f"Specialist Agent Profiles / {agent_name}"
    elif rel.startswith('directives/'):
        return f"Core Directives / {file_path.stem}"
    elif rel.startswith('docs/'):
        sub = " / ".join(parts[1:-1]) if len(parts) > 2 else "Core Docs"
        return f"Architecture & Documentation / {sub} / {file_path.stem}"
    elif rel.startswith('wiki/'):
        return f"Knowledge Base & Wiki / {file_path.stem}"
    elif rel.startswith('src/data/knowledge/'):
        cat = parts[3] if len(parts) > 3 else "Knowledge"
        return f"Domain Knowledge Repository / {cat} / {file_path.stem}"
    else:
        return f"Documentation / {file_path.stem}"

def extract_meta_fields(banner_content: str) -> dict:
    meta = {
        "docs": None,
        "failure_path": None,
        "telemetry": None,
        "persona": None,
    }
    
    # @docs
    docs_match = re.search(r'-\s*(?:@docs|\*\*@docs\*\*)\s+([A-Za-z0-9_:\-/]+)', banner_content)
    if docs_match:
        meta["docs"] = docs_match.group(1).strip()
    
    # Failure Path
    fail_match = re.search(r'-\s*\*\*Failure Path\*\*:\s*([^\n\r]+)', banner_content)
    if fail_match:
        meta["failure_path"] = fail_match.group(1).strip()
        
    # Telemetry
    tel_match = re.search(r'-\s*\*\*Telemetry Link\*\*:\s*(?:Search\s+)?(`\[[^\]]+\]`|\[[^\]]+\])', banner_content)
    if tel_match:
        meta["telemetry"] = tel_match.group(1).strip('`')
    else:
        # Check for inline tag search
        tag_match = re.search(r'\[([A-Z0-9_\-]+)\]', banner_content)
        if tag_match and tag_match.group(1) not in {'IMPORTANT', 'NOTE', 'TIP', 'WARNING', 'CAUTION'}:
            meta["telemetry"] = f"[{tag_match.group(1)}]"
            
    # Persona directive if any
    if "### 🤖 AI Persona Directive" in banner_content:
        persona_match = re.search(r'### 🤖 AI Persona Directive\s*\n>([^\n#]+)', banner_content)
        if persona_match:
            meta["persona"] = persona_match.group(1).strip()
            
    return meta

def build_modern_header(subsystem: str, meta: dict) -> str:
    lines = [
        "> [!IMPORTANT]",
        "> **AI Context & Knowledge Heritage**",
        f"> - **Subsystem**: {subsystem}",
    ]
    
    if meta.get("docs"):
        lines.append(f"> - **Architecture**: `@docs {meta['docs']}`")
    else:
        lines.append("> - **Architecture**: `@docs ARCHITECTURE:Documentation`")
        
    if meta.get("failure_path"):
        lines.append(f"> - **Failure Path**: {meta['failure_path']}")
    else:
        lines.append("> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.")
        
    if meta.get("telemetry"):
        lines.append(f"> - **Observability**: Traceability via `execution/parity_guard.py` (`{meta['telemetry']}`)")
    else:
        lines.append("> - **Observability**: Traceability via `execution/parity_guard.py`")
        
    if meta.get("persona"):
        lines.append(">\n> ### 🤖 AI Persona Directive")
        lines.append(f"> {meta['persona']}")
        
    return "\n".join(lines)

def migrate_markdown_file(file_path: Path) -> bool:
    try:
        content = file_path.read_text(encoding='utf-8')
    except Exception as e:
        print(f"[ERR] Failed to read {file_path}: {e}")
        return False
        
    subsystem = derive_subsystem(file_path)
    
    # 1. Extract optional YAML frontmatter
    yaml_frontmatter = ""
    rest = content
    yaml_match = re.match(r'^(---\r?\n[\s\S]*?\r?\n---\r?\n)', content)
    if yaml_match:
        yaml_frontmatter = yaml_match.group(1)
        rest = content[len(yaml_frontmatter):]
        
    # Check if this file has a legacy callout banner
    # Patterns to catch:
    # > [!IMPORTANT]
    # > **AI Assist Note (Knowledge Heritage)**: ...
    # or consecutive duplicated > [!IMPORTANT] blocks
    legacy_banner_match = re.match(
        r'^\s*(?:>\s*\[!IMPORTANT\][\s\S]*?(?:Traceability via `parity_guard\.py`\.?|\[\w+\] in audit logs\.?)\s*\r?\n)+\s*',
        rest
    )
    
    if not legacy_banner_match:
        # Check for standalone single-block AI Assist Note
        legacy_banner_match = re.match(
            r'^\s*>\s*\[!IMPORTANT\][\s\S]*?(?:### 🔍 Debugging & Observability[\s\S]*?Traceability[^\n]+)\s*\r?\n',
            rest
        )
        
    if legacy_banner_match:
        banner_raw = legacy_banner_match.group(0)
        after_banner = rest[len(banner_raw):].lstrip('\r\n')
        
        meta = extract_meta_fields(banner_raw)
        modern_banner = build_modern_header(subsystem, meta)
        
        # Also check and strip legacy metadata footers at the end of file (e.g. [//]: # (Metadata: [AGENTS]))
        after_banner = re.sub(r'\r?\n*\[//\]:\s*#\s*\(Metadata:\s*\[[^\]]+\]\)\s*$', '', after_banner)
        
        new_content = ""
        if yaml_frontmatter:
            new_content = yaml_frontmatter + "\n" + modern_banner + "\n\n" + after_banner
        else:
            new_content = modern_banner + "\n\n" + after_banner
            
        if new_content != content:
            file_path.write_text(new_content, encoding='utf-8')
            return True
            
    return False

def main():
    target_dir = ROOT
    if len(sys.argv) > 1:
        target_dir = Path(sys.argv[1]).resolve()
        
    print(f"🚀 [migrate_markdown_headers] Scanning: {target_dir}")
    
    total = 0
    migrated = 0
    
    for dirpath, dirnames, filenames in os.walk(target_dir):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for f in filenames:
            if not f.endswith('.md'):
                continue
                
            fp = Path(dirpath) / f
            total += 1
            if migrate_markdown_file(fp):
                migrated += 1
                rel = fp.relative_to(ROOT).as_posix()
                print(f"[MIGRATED] {rel}")
                
    print(f"\n📊 Summary: {total} markdown files scanned, {migrated} upgraded to Sovereign Standard.")

if __name__ == "__main__":
    main()
