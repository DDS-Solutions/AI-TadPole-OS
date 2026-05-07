"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[parse_coverage]` in system logs.
"""

import json
import os

coverage_path = r"d:\TadpoleOS-Dev\coverage\coverage-final.json"

if not os.path.exists(coverage_path):
    print("Coverage report not found.")
else:
    with open(coverage_path, "r") as f:
        data = json.load(f)
        
    # Vitest coverage-final.json is usually a mapping of file paths to coverage data
    # The coverage data usually has "s" (statements), "f" (functions), "b" (branches)
    
    summary = {}
    for file_path, file_data in data.items():
        # This format is for Istanbul/NYC which Vitest uses
        # We want to find files with low coverage
        # Actually, let's just count files with 0 coverage or low coverage
        
        # In this format, file_data['s'] is a dict of statement index to count
        s = file_data.get('s', {})
        total_s = len(s)
        covered_s = sum(1 for count in s.values() if count > 0)
        
        pct = (covered_s / total_s * 100) if total_s > 0 else 0
        summary[file_path] = pct

    # Sort by coverage percentage
    sorted_summary = sorted(summary.items(), key=lambda x: x[1])
    
    print(f"Total files in coverage report: {len(summary)}")
    print("\nFiles with LOWEST coverage (first 20):")
    for file_path, pct in sorted_summary[:20]:
        print(f"{pct:.2f}% - {file_path}")

# Metadata: [parse_coverage]
