#!/usr/bin/env python3
import os
import sys
import argparse
import io
from pathlib import Path

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

# --- Configuration ---
TARGET_DIRS = [
    'server-rs/src',
    'src',
    'src-tauri',
    'wasm-codec',
    'docs'
]

FILE_EXTENSIONS = {
    '.rs': 'rust',
    '.ts': 'typescript',
    '.tsx': 'typescript',
    '.md': 'markdown',
    '.css': 'css',
    '.html': 'html'
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
    'docs': 'Documentation'
}

def get_pillar(file_path):
    rel_path = file_path.replace('\\', '/')
    for k, v in PILLAR_MAP.items():
        if k in rel_path:
            return v
    return 'Core'

def get_headers(file_path, pillar):
    basename = os.path.basename(file_path)
    ext = os.path.splitext(file_path)[1]
    
    if ext == '.rs':
        return f"""//! @docs ARCHITECTURE:{pillar}
//! 
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[{basename}]` in tracing logs.

"""
    elif ext in ['.ts', '.tsx']:
        return f"""/**
 * @docs ARCHITECTURE:{pillar}
 * 
 * ### AI Assist Note
 * **Sovereign Interface component for the Tadpole OS dashboard.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[{basename}]` in observability traces.
 */

"""
    elif ext == '.md':
        return f"""> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:{pillar}**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

"""
    elif ext == '.html':
        return f"""<!--
  @docs ARCHITECTURE:{pillar}
  ### AI Assist Note
  Standardized HTML resource.
  ### 🔍 Debugging & Observability
  Validated via Sovereign Audit.
-->

"""
    else:
        return f"""/*
 * @docs ARCHITECTURE:{pillar}
 * ### AI Assist Note
 * Standardized resource for the Tadpole OS engine.
 * ### 🔍 Debugging & Observability
 * Validated via Sovereign Audit.
 */

"""

def process_file(file_path, audit_only=False):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            
        missing = [m for m in REQUIRED_MARKERS if m not in content]
        if not missing:
            return "ALREADY_AWAKENED"
            
        if audit_only:
            return "MISSING"
            
        pillar = get_pillar(file_path)
        header = get_headers(file_path, pillar)
        
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(header + content)
            
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
            if any(part in r for part in ['.git', 'node_modules', 'target', 'dist', '.tmp']):
                continue
                
            for f in files:
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
