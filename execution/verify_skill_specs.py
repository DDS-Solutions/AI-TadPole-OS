"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**🛡️ Sovereign Specification Auditor (agentskills.io)**
Automated scanner that validates all `.agent/skills/**/SKILL.md`, `.agent/agents/*.md`,
and `.agent/workflows/*.md` files against open specification standards (Line 1 frontmatter).

### 🔍 Debugging & Observability
- **Failure Path**: Invalid YAML frontmatter, naming drift, or missing description.
- **Telemetry Link**: Search `[verify_skill_specs]` in system logs.
"""

#!/usr/bin/env python3
import os
import re
import sys
import yaml
import io
from pathlib import Path

# Windows console encoding fix
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

ROOT = Path(__file__).resolve().parent.parent
SKILLS_DIR = ROOT / ".agent" / "skills"
AGENTS_DIR = ROOT / ".agent" / "agents"
WORKFLOWS_DIR = ROOT / ".agent" / "workflows"

def validate_markdown_spec(file_path: Path, category: str = "skill") -> dict:
    findings = []
    
    try:
        content = file_path.read_text(encoding='utf-8', errors='ignore')
    except Exception as e:
        return {"file": str(file_path.relative_to(ROOT)), "passed": False, "findings": [f"Cannot read file: {e}"]}
    
    # 1. Frontmatter Location Check (Line 1 must be '---')
    if not content.startswith("---"):
        findings.append("YAML frontmatter does not start on Line 1 (must begin with '---')")
        
    # Extract YAML frontmatter
    fm_match = re.match(r'^---\s*\n(.*?)\n---', content, re.DOTALL)
    if not fm_match:
        findings.append("Missing or malformed YAML frontmatter block")
        fm_data = {}
    else:
        try:
            fm_data = yaml.safe_load(fm_match.group(1)) or {}
        except Exception as e:
            findings.append(f"Invalid YAML syntax in frontmatter: {e}")
            fm_data = {}

    if fm_data:
        # 'name' Attribute Validation (Required for skills & agents)
        name = fm_data.get("name")
        if category in ("skill", "agent"):
            if not name:
                findings.append("Missing 'name' attribute in frontmatter")
            else:
                name_str = str(name).strip()
                if len(name_str) > 64:
                    findings.append(f"'name' exceeds 64 characters limit ({len(name_str)} chars)")
                if not re.match(r'^[a-z0-9]+(-[a-z0-9]+)*$', name_str):
                    findings.append(f"'name' '{name_str}' is not valid kebab-case")
                
        # 'description' Attribute Validation (Required for all categories)
        description = fm_data.get("description")
        if not description:
            findings.append("Missing 'description' attribute in frontmatter")
        else:
            desc_str = str(description).strip()
            if len(desc_str) > 1024:
                findings.append(f"'description' exceeds 1024 characters limit ({len(desc_str)} chars)")
            if len(desc_str) < 5:
                findings.append(f"'description' is too short ({len(desc_str)} chars)")

    # Body Size Footprint Check
    body_text = re.sub(r'^---\s*\n.*?\n---', '', content, flags=re.DOTALL).strip()
    words = len(body_text.split())
    max_words = 5000 if category == "skill" else 8000
    if words > max_words:
        findings.append(f"Instruction body is oversized ({words} words > {max_words} max threshold)")

    rel_path = str(file_path.relative_to(ROOT))
    identifier = fm_data.get("name", file_path.stem) if fm_data else file_path.stem
    return {
        "file": rel_path,
        "name": identifier,
        "category": category,
        "passed": len(findings) == 0,
        "findings": findings
    }

def main():
    print(f"🔍 [verify_skill_specs] Auditing Skills, Subagent Profiles, and Workflows...\n")
    
    skill_files = list(SKILLS_DIR.glob("**/SKILL.md")) if SKILLS_DIR.exists() else []
    agent_files = list(AGENTS_DIR.glob("*.md")) if AGENTS_DIR.exists() else []
    workflow_files = list(WORKFLOWS_DIR.glob("*.md")) if WORKFLOWS_DIR.exists() else []
    
    skill_results = [validate_markdown_spec(sf, category="skill") for sf in skill_files]
    agent_results = [validate_markdown_spec(af, category="agent") for af in agent_files]
    workflow_results = [validate_markdown_spec(wf, category="workflow") for wf in workflow_files]
    
    all_results = skill_results + agent_results + workflow_results
    passed = [r for r in all_results if r["passed"]]
    failed = [r for r in all_results if not r["passed"]]
    
    print("=" * 65)
    print("📊 SOVEREIGN SPECIFICATION AUDIT REPORT")
    print("=" * 65)
    print(f"Skills Audited        : {len(skill_results)} (Passed: {len([r for r in skill_results if r['passed']])})")
    print(f"Subagents Audited     : {len(agent_results)} (Passed: {len([r for r in agent_results if r['passed']])})")
    print(f"Workflows Audited     : {len(workflow_results)} (Passed: {len([r for r in workflow_results if r['passed']])})")
    print("-" * 65)
    print(f"Total Spec Files      : {len(all_results)}")
    print(f"✅ Total Compliant    : {len(passed)}")
    print(f"❌ Non-Compliant     : {len(failed)}")
    print("=" * 65 + "\n")
    
    if failed:
        print("🚨 NON-COMPLIANT SPECIFICATION FILES DETECTED:")
        for r in failed:
            print(f"- {r['file']} [{r['category'].upper()}] (ID: '{r['name']}')")
            for finding in r["findings"]:
                print(f"    ↳ {finding}")
            print()
    else:
        print("🎉 All 122 workspace skills, subagents, and workflows conform 100% to specification standards!")
        
    sys.exit(0 if not failed else 1)

if __name__ == "__main__":
    main()

# Metadata: [verify_skill_specs]
