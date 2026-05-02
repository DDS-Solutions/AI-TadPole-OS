"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[audit_manual]` in system logs.
"""

import os
import json
import re
from pathlib import Path

def audit_manual():
    workspace_root = Path("D:/TadpoleOS-Dev")
    manual_path = workspace_root / "docs/OPERATIONS_MANUAL.md"
    
    if not manual_path.exists():
        print(f"Error: Manual not found at {manual_path}")
        return

    manual_content = manual_path.read_text(encoding="utf-8")
    
    # 1. Discover Skills
    skills = []
    # Native
    native_dir = workspace_root / "server-rs/data/skills"
    for d in native_dir.iterdir():
        if d.is_dir() and (d / "skill.json").exists():
            skills.append(d.name)
    
    # Script
    execution_dir = workspace_root / "execution"
    for f in execution_dir.glob("*.json"):
        skills.append(f.stem)
        
    # 2. Check for Skill mentions in manual
    missing_skills = []
    for skill in skills:
        if skill not in manual_content:
            missing_skills.append(skill)
            
    # 3. Check for specific sections
    sections = [
        "Governance",
        "Communication Hub",
        "Registry Hub",
        "Security Hub",
        "Tool Reference"
    ]
    missing_sections = [s for s in sections if f"## {s}" not in manual_content]
    
    # 4. Generate Report
    report = [
        "# Documentation Drift Report",
        "",
        "## Summary",
        f"Audited: `OPERATIONS_MANUAL.md`",
        f"Date: 2026-04-11",
        "",
        "## Gaps Found",
        "",
        "### Missing Skill Documentation",
        "The following active skills are registered in the OS but NOT mentioned in the manual:"
    ]
    
    if missing_skills:
        for s in missing_skills:
            report.append(f"- [ ] `{s}`")
    else:
        report.append("- No missing skills found.")
        
    report.append("")
    report.append("### Missing Architectural Sections")
    if missing_sections:
        for s in missing_sections:
            report.append(f"- [ ] `## {s}`")
    else:
        report.append("- All core sections present.")
        
    report_path = workspace_root / "docs/documentation_drift_report.md"
    report_path.write_text("\n".join(report), encoding="utf-8")
    
    print(f"Report generated at {report_path}")

if __name__ == "__main__":
    audit_manual()

# Metadata: [audit_manual]
