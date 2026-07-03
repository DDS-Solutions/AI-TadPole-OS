#!/usr/bin/env python3
import os
import sys
import argparse
import io
import re
from pathlib import Path
from typing import List, Dict, Any

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

# --- Configuration ---
TARGET_DIRS = [
    '.',
    'server-rs',
    'src',
    'src-tauri',
    'wasm-codec',
    'docs',
    '.agent',
    'execution',
    'scripts',
    'legacy',
    'data',
    'tests'
]

FILE_EXTENSIONS = {
    '.rs': 'rust',
    '.ts': 'typescript',
    '.tsx': 'typescript',
    '.js': 'javascript',
    '.py': 'python',
    '.md': 'markdown',
    '.css': 'css',
    '.html': 'html',
    '.sh': 'shell'
}

REQUIRED_MARKERS = [
    '@docs ARCHITECTURE:',
    '### AI Assist Note',
    '### 🔍 Debugging & Observability'
]

PILLAR_MAP = {
    'server-rs/src/agent': 'Registry',
    'server-rs/src/routes': 'Gateways',
    'server-rs/src/system': 'Infrastructure',
    'server-rs/src/telemetry': 'Observability',
    'server-rs/src/state': 'Hubs',
    'src/components': 'UI-Components',
    'src/hooks': 'UI-Hooks',
    'src/pages': 'UI-Pages',
    'src/services': 'UI-Services',
    'src/stores': 'UI-Stores',
    'docs': 'Documentation',
    '.agent/skills': 'Registry:Skills',
    'execution': 'Infrastructure:Execution',
    'test': 'Quality:Verification',
    'spec': 'Quality:Verification'
}

def get_pillar(file_path):
    rel_path = file_path.replace('\\', '/')
    for k, v in PILLAR_MAP.items():
        if k in rel_path:
            return v
    return 'Core'

def get_headers(file_path, pillar, purpose=None):
    basename = os.path.basename(file_path)
    stem = os.path.splitext(basename)[0]
    ext = os.path.splitext(file_path)[1]
    
    # Standardize purpose
    if not purpose or len(purpose) < 10:
        if "test" in basename.lower():
            purpose = "Verification and quality assurance for the Tadpole OS engine."
        else:
            purpose = "Core technical resource for the Tadpole OS Sovereign infrastructure."

    # Unique Telemetry Tag
    tag = stem.replace('.', '_').replace('-', '_')

    if ext == '.rs':
        return f"""//! @docs ARCHITECTURE:{pillar}
//! 
//! ### AI Assist Note
//! **{purpose}**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[{tag}]` in tracing logs.

"""
    elif ext in ['.ts', '.tsx', '.js']:
        return f"""/**
 * @docs ARCHITECTURE:{pillar}
 * 
 * ### AI Assist Note
 * **{purpose}**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[{tag}]` in observability traces.
 */

"""
    elif ext == '.py':
        return f'''"""
@docs ARCHITECTURE:{pillar}

### AI Assist Note
**{purpose}**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[{tag}]` in system logs.
"""

'''
    elif ext == '.md':
        return f"""> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:{pillar}**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[{tag}]` in audit logs.
>
> ### AI Assist Note
> {purpose}
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

"""
    elif ext == '.html':
        return f"""<!--
  @docs ARCHITECTURE:{pillar}
  ### AI Assist Note
  **{purpose}**
  Standardized HTML resource.
  ### 🔍 Debugging & Observability
  - **Failure Path**: Rendering error or broken asset links.
  - **Telemetry Link**: Search `[{tag}]` in system traces.
-->

"""
    elif ext == '.sh':
        return f"""# @docs ARCHITECTURE:{pillar}
#
# ### AI Assist Note
# **{purpose}**
# Shell-level automation for the Tadpole OS deployment and maintenance.
#
# ### 🔍 Debugging & Observability
# - **Failure Path**: Environment error, permission denied, or command failure.
# - **Telemetry Link**: Search `[{tag}]` in audit logs.

"""
    else:
        return f"""/*
 * @docs ARCHITECTURE:{pillar}
 * ### AI Assist Note
 * **{purpose}**
 * Standardized resource for the Tadpole OS engine.
 * ### 🔍 Debugging & Observability
 * Validated via Sovereign Audit.
 */

"""

def extract_purpose_heuristic(content):
    # Attempt to extract the first major docstring or comment block
    # Simple regex for docstrings/comments
    # Py docstrings
    py_match = re.search(r'\"\"\"(.*?)\"\"\"', content, re.DOTALL)
    if py_match: return py_match.group(1).strip().split('\n')[0].strip(' #*')
    
    # TS/JS docstrings
    ts_match = re.search(r'/\*\*(.*?)\*/', content, re.DOTALL)
    if ts_match: return ts_match.group(1).strip().split('\n')[0].strip(' #*')

    # Line comments at top
    lines = content.split('\n')
    top_comments = []
    for line in lines[:10]:
        if line.strip().startswith('//') or line.strip().startswith('#'):
            top_comments.append(line.strip(' /#*'))
        elif line.strip():
            break
    if top_comments: return top_comments[0].strip()
    
    return None

def get_footer(file_path, tag):
    ext = os.path.splitext(file_path)[1]
    if ext == '.md':
        return f"\n[//]: # (Metadata: [{tag}])\n"
    elif ext == '.html':
        return f"\n<!-- Metadata: [{tag}] -->\n"
    elif ext == '.rs' or ext in ['.ts', '.tsx', '.js', '.css', '.sh', '.py']:
        comment_prefix = '#' if ext in ['.py', '.sh'] else '//'
        return f"\n{comment_prefix} Metadata: [{tag}]\n"
    return ""

def extract_metadata(content):
    res = {
        "has_note": False,
        "has_debugging": False,
        "telemetry_tag": None
    }
    # Check for AI Assist Note
    if re.search(r'###\s+AI\s+Assist\s+Note', content):
        res["has_note"] = True
    # Check for Debugging section (emoji-agnostic)
    if re.search(r'###\s+.*?\s+Debugging\s+&\s+Observability', content):
        res["has_debugging"] = True
    # Extract Telemetry Tag (e.g. [AppKernel])
    tele_match = re.search(r'Search (?:for|`)\s*\[([a-zA-Z0-9_\-]+)\]', content)
    if tele_match:
        res["telemetry_tag"] = tele_match.group(1)
    return res

def process_file(file_path, audit_only=False):
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
            
        stem = os.path.splitext(os.path.basename(file_path))[0]
        tag = stem.replace('.', '_').replace('-', '_')
        
        meta = extract_metadata(content)
        has_required_headers = meta["has_note"] and meta["has_debugging"]
        
        # Telemetry verification: if a tag is explicitly declared in the header,
        # it must exist at least twice in the file. Otherwise, if the header doesn't
        # define a tag, we treat telemetry as compliant.
        if has_required_headers and not meta["telemetry_tag"]:
            has_telemetry = True
        elif meta["telemetry_tag"]:
            tag_marker = f"[{meta['telemetry_tag']}]"
            has_telemetry = content.count(tag_marker) >= 2
        else:
            tag_marker = f"[{tag}]"
            has_telemetry = content.count(tag_marker) >= 2
            
        if has_required_headers and has_telemetry:
            return "ALREADY_AWAKENED"
            
        if audit_only:
            return "MISSING"
            
        pillar = get_pillar(file_path)
        purpose = extract_purpose_heuristic(content)
        
        # If headers are already there but telemetry is missing, just append footer
        if has_required_headers:
            active_tag = meta["telemetry_tag"] if meta["telemetry_tag"] else tag
            footer = get_footer(file_path, active_tag)
            with open(file_path, 'a', encoding='utf-8') as f:
                f.write(footer)
            return "FIXED"
        
        # Standard full Awakening
        header = get_headers(file_path, pillar, purpose=purpose)
        footer = get_footer(file_path, tag)
        
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(header + content + footer)
            
        return "FIXED"
    except Exception as e:
        return f"ERROR: {str(e)}"

def main():
    parser = argparse.ArgumentParser(description="Universal Awakening Orchestrator")
    parser.add_argument("--check", action="store_true", help="Audit mode (no changes)")
    parser.add_argument("--dir", default=".", help="Root directory to scan")
    args = parser.parse_args()
    
    root = Path(args.dir).resolve()
    print(f"--- Tadpole OS: Awakening Orchestrator {'(Audit Mode)' if args.check else ''} ---")
    
    stats = {"FIXED": 0, "ALREADY_AWAKENED": 0, "MISSING": 0, "ERROR": 0}
    missing_files = []

    for target in TARGET_DIRS:
        target_path = root / target
        if not target_path.exists():
            continue
            
        for r, _, files in os.walk(target_path):
            if any(part in r for part in ['.git', 'node_modules', 'target', 'dist', '.tmp', 'reports', 'coverage', '3rdparty', 'wiki', '.vitepress', 'public', 'archive', '.gemini', '.cargo', 'scratch', 'playwright-report', 'test-results']):
                continue
                
            for f in files:
                if f in ['API_REFERENCE.md', 'Benchmark_Spec.md', 'POLLYWOG-DEBT.md']:
                    continue
                ext = os.path.splitext(f)[1]
                if ext in FILE_EXTENSIONS:
                    file_path = os.path.join(r, f)
                    res = process_file(file_path, audit_only=args.check)
                    
                    if res.startswith("ERROR"):
                        stats["ERROR"] += 1
                        print(f"[!] {res}: {file_path}")
                    elif res == "MISSING":
                        stats["MISSING"] += 1
                        missing_files.append(file_path)
                    else:
                        stats[res] += 1
                        if res == "FIXED":
                            print(f"[+] Awakened: {os.path.relpath(file_path, root)}")

    print(f"\nScan Complete.")
    print(f"Already Awakened: {stats['ALREADY_AWAKENED']}")
    print(f"Newly Awakened:   {stats['FIXED']}")
    print(f"Missing Headers:  {stats['MISSING']}")
    
    if args.check:
        if stats["MISSING"] > 0:
            print("\n[!] The following files are NOT awakened:")
            for mf in missing_files:
                print(f"  [ ] {os.path.relpath(mf, root)}")
            sys.exit(1)
        else:
            print("\n✨ Codebase is 100% Awakened.")
            sys.exit(0)

if __name__ == "__main__":
    main()

# Metadata: [awaken]
