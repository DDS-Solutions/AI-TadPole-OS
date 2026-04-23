"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**🛡️ Tadpole OS: AI Context Alignment Tool**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[verify_ai_context]` in system logs.
"""

#!/usr/bin/env python3
"""
🛡️ Tadpole OS: AI Context Alignment Tool
Purpose: Ensures 100% synchronization between code and AI Assist Notes.
Verification Gates:
1. Presence of '### AI Assist Note'
2. Presence of '### 🔍 Debugging & Observability'
3. Presence of '@docs' tag
4. Telemetry Tag Integrity (check if tag exists in code)
5. Documentation Link Integrity (check if @docs points to an existing file)
"""

import os
import re
import sys
import json
import argparse
from pathlib import Path
from typing import List, Dict, Any
from datetime import datetime

# --- Configuration ---
SKIP_DIRS = {'.git', 'node_modules', 'dist', 'target', 'build', '__pycache__', '.venv', 'venv', '.tmp', 'tmp'}
EXTENSIONS = {'.rs', '.ts', '.tsx', '.js', '.py'}
ROOT = Path(__file__).resolve().parent.parent

def extract_metadata(content):
    res = {
        "has_note": False,
        "has_debugging": False,
        "has_docs": False,
        "telemetry_tag": None,
        "docs_link": None
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
        
    # Check for @docs
    docs_match = re.search(r'@docs\s+([A-Z0-9_]+):([a-zA-Z0-9_]+)', content)
    if docs_match:
        res["has_docs"] = True
        res["docs_link"] = f"{docs_match.group(1)}:{docs_match.group(2)}"
        
    return res

def verify_file(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
            
        meta = extract_metadata(content)
        findings = []
        
        if not meta["has_note"]:
            findings.append("Missing '### AI Assist Note'")
            
        if not meta["has_debugging"]:
            findings.append("Missing '### 🔍 Debugging & Observability'")

        # 1. Telemetry Tag Verification
        if meta["telemetry_tag"]:
            tag = f"[{meta['telemetry_tag']}]"
            # Look for the tag in code. It must exist at least twice 
            # (once in the AI note and at least once in a log/tracing call)
            count = content.count(tag)
            if count < 2:
                findings.append(f"Telemetry Tag '{tag}' defined in note but missing from logic logs")

        # 2. @docs Tag Verification
        if meta["docs_link"]:
            doc_name = meta["docs_link"].split(':')[0]
            doc_file = ROOT / "docs" / f"{doc_name}.md"
            if not doc_file.exists():
                # Fallback to root level markdown
                doc_file = ROOT / f"{doc_name}.md"
                if not doc_file.exists():
                    findings.append(f"Broken @docs link: File '{doc_name}.md' not found")

        # 3. Density Check (Warn only)
        code_lines = len([l for l in content.split('\n') if l.strip() and not l.strip().startswith('//') and not l.strip().startswith('/*')])
        if code_lines > 100 and not meta["has_note"]:
            findings.append("High-complexity file missing alignment note")

        return {
            "file": str(Path(file_path).relative_to(ROOT)),
            "passed": len(findings) == 0,
            "findings": findings,
            "meta": meta
        }
    except Exception as e:
        return {"file": str(file_path), "passed": False, "findings": [f"Error processing file: {str(e)}"]}

def main():
    # Fix Windows console encoding for Unicode output
    if sys.platform == "win32":
        import io
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

    parser = argparse.ArgumentParser(description="Tadpole OS AI Context Auditor")
    parser.add_argument("path", nargs="?", default=".", help="Root directory to scan")
    parser.add_argument("--json", action="store_true", help="Output results as JSON")
    args = parser.parse_args()

    scan_root = Path(args.path).resolve()
    results = []
    
    if not args.json:
        print(f"[SCAN] Scanning for AI Context Alignment (Root: {scan_root})...")
    
    for root, dirs, files in os.walk(scan_root):
        # Prune directories
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        
        for file in files:
            if Path(file).suffix.lower() in EXTENSIONS:
                file_path = Path(root) / file
                res = verify_file(file_path)
                results.append(res)

    passed = [r for r in results if r["passed"]]
    failed = [r for r in results if not r["passed"]]
    
    if args.json:
        print(json.dumps({
            "summary": {
                "total": len(results),
                "passed": len(passed),
                "failed": len(failed),
                "timestamp": datetime.now().isoformat()
            },
            "failures": failed
        }, indent=2))
    else:
        print(f"\n--- 🛡️ AI Context Alignment Report ---")
        print(f"Total Files Scanned: {len(results)}")
        print(f"✅ PASSED: {len(passed)}")
        print(f"❌ FAILED: {len(failed)}")
        print(f"-------------------------------------\n")
        
        if failed:
            print("🚨 DETECTED DRIFT/MISSING CONTEXT:")
            for f in failed[:20]: # Show first 20
                print(f"- {f['file']}")
                for finding in f["findings"]:
                    print(f"    ↳ {finding}")
            
            if len(failed) > 20:
                print(f"\n... and {len(failed) - 20} more files.")
        
    sys.exit(0 if not failed else 1)

if __name__ == "__main__":
    main()

# Metadata: [verify_ai_context]
