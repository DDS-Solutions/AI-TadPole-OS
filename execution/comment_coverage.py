"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Sovereign Documentation Metric Engine**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[comment_coverage]` in system logs.
"""

"""
Sovereign Documentation Metric Engine

### AI Assist Note
**Audit Pillar**: Calculates the codebase-wide comment density benchmark. 
Supports **TS/RS/PY** parsing with support for block comments and 
docstrings. Provides the primary metric for the "Sovereign Documentation 
Sprint" compliance gate.

### 🔍 Debugging & Observability
- **Metric Logic**: Coverage = (Comment Lines) / (Comment Lines + Code Lines) * 100.
- **Trace Scope**: `execution::comment_coverage`
"""
import os
import json
from pathlib import Path
from typing import List, Dict, Any

def analyze_file(file_path):
    """Calculates granular line statistics for a single file."""
    ext = file_path.suffix.lower()
    if ext not in ['.ts', '.tsx', '.rs', '.py', '.js']:
        return None
        
    try:
        lines = file_path.read_text(encoding='utf-8', errors='replace').splitlines()
    except Exception:
        return None

    stats = {"total": len(lines), "comment": 0, "blank": 0}
    in_block = False
    block_char = None 

    for line in lines:
        s = line.strip()
        if not s:
            stats["blank"] += 1
            continue

        if ext == '.py':
            if not in_block:
                if s.startswith('#'):
                    stats["comment"] += 1
                elif '"""' in s:
                    stats["comment"] += 1
                    if s.count('"""') == 1:
                        in_block = True
                        block_char = '"""'
                elif "'''" in s:
                    stats["comment"] += 1
                    if s.count("'''") == 1:
                        in_block = True
                        block_char = "'''"
            else:
                stats["comment"] += 1
                if block_char in s:
                    in_block = False
            continue

        if ext in ['.ts', '.tsx', '.rs', '.js']:
            if not in_block:
                if s.startswith('//'):
                    stats["comment"] += 1
                elif s.startswith('/*'):
                    stats["comment"] += 1
                    if '*/' not in s:
                        in_block = True
            else:
                stats["comment"] += 1
                if '*/' in s:
                    in_block = False
            continue
            
    stats["code"] = stats["total"] - stats["blank"] - stats["comment"]
    return stats

def run_coverage():
    workspace = Path(".")
    dirs_to_scan = [
        "src",
        "server-rs",
        "execution",
        "src-tauri",
        "wasm-codec",
        "scripts"
    ]
    
    exclude_dirs = {"target", "node_modules", "dist", ".git", "build"}
    
    report = {}
    
    for d_name in dirs_to_scan:
        d_path = workspace / d_name
        if not d_path.exists():
            continue
            
        dir_stats = {"files": 0, "total": 0, "comment": 0, "blank": 0, "code": 0, "details": []}
        
        for file_path in d_path.rglob("*"):
            # Skip heavy or irrelevant directories
            if any(part in exclude_dirs for part in file_path.parts):
                continue
                
            if file_path.is_file():
                res = analyze_file(file_path)
                if res:
                    dir_stats["files"] += 1
                    for k in ["total", "comment", "blank", "code"]:
                        dir_stats[k] += res[k]
                    
                    # Add file details for deep analysis
                    denom = res["comment"] + res["code"]
                    cov = (res["comment"] / denom * 100) if denom > 0 else 0
                    dir_stats["details"].append({
                        "path": str(file_path),
                        "coverage": cov,
                        "code": res["code"]
                    })
        
        if dir_stats["files"] > 0:
            # Sort details by coverage (ascending) to find leanest files
            dir_stats["details"].sort(key=lambda x: x["coverage"])
            # Keep top 10 leanest per dir to save memory
            dir_stats["details"] = dir_stats["details"][:10]
            
            denominator = dir_stats["comment"] + dir_stats["code"]
            dir_stats["coverage"] = (dir_stats["comment"] / denominator * 100) if denominator > 0 else 0
            report[d_name] = dir_stats
            
    # Calculate Grand Total
    total_stats = {"files": 0, "total": 0, "comment": 0, "blank": 0, "code": 0}
    for d in report.values():
        for k in ["files", "total", "comment", "blank", "code"]:
            total_stats[k] += d[k]
            
    total_denom = total_stats["comment"] + total_stats["code"]
    total_stats["coverage"] = (total_stats["comment"] / total_denom * 100) if total_denom > 0 else 0
    report["OVERALL"] = total_stats
    
    print(json.dumps(report, indent=2))

if __name__ == "__main__":
    run_coverage()

# Metadata: [comment_coverage]
