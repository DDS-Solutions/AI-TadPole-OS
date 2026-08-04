"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**🛡️ Specification Frontmatter Alignment Utility**
Auto-fixes frontmatter placement across `.agent/skills/`, `.agent/agents/`, and `.agent/workflows/`
to ensure 100% compliance with open specification standards (YAML block at Line 1).

### 🔍 Debugging & Observability
- **Failure Path**: Regex mismatch or file write permissions.
- **Telemetry Link**: Search `[fix_agent_frontmatter]` in system logs.
"""

#!/usr/bin/env python3
import os
import re
import sys
import io
import yaml
from pathlib import Path

# Windows console encoding fix
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

ROOT = Path(__file__).resolve().parent.parent
AGENTS_DIR = ROOT / ".agent" / "agents"
WORKFLOWS_DIR = ROOT / ".agent" / "workflows"

def align_markdown_frontmatter(file_path: Path) -> bool:
    content = file_path.read_text(encoding='utf-8', errors='ignore')
    
    # If frontmatter already starts at Line 1 and is valid, skip
    if content.startswith("---"):
        fm_match = re.match(r'^---\s*\n(.*?)\n---', content, re.DOTALL)
        if fm_match:
            try:
                fm_data = yaml.safe_load(fm_match.group(1))
                if isinstance(fm_data, dict) and ("name" in fm_data or "description" in fm_data):
                    return False
            except Exception:
                pass

    # Clean pre-existing noise like `>--- File: filename.md ---`
    content_clean = re.sub(r'^>\s*---\s*File:.*?\n', '', content, flags=re.MULTILINE)

    # Search for valid frontmatter block (description: or name:)
    fm_match = re.search(r'---\s*\n((?:name|description):\s*.*?\n---)', content_clean, re.DOTALL)
    if fm_match:
        fm_block = fm_match.group(1).strip()
        pre_fm = content_clean[:fm_match.start()].strip()
        post_fm = content_clean[fm_match.end():].strip()
        
        new_content = f"---\n{fm_block}\n\n"
        if pre_fm:
            new_content += f"{pre_fm}\n\n"
        if post_fm:
            new_content += f"{post_fm}\n"
    else:
        file_stem = file_path.stem
        desc = f"Sovereign workflow procedure for /{file_stem}."
        clean_body = content_clean.strip()
        new_content = f"---\ndescription: \"{desc}\"\n---\n\n{clean_body}\n"

    file_path.write_text(new_content, encoding='utf-8')
    return True

def main():
    print(f"🔧 [fix_agent_frontmatter] Aligning frontmatter in agents & workflows...")
    
    target_files = list(AGENTS_DIR.glob("*.md")) + list(WORKFLOWS_DIR.glob("*.md"))
    fixed_count = 0
    for tf in target_files:
        if align_markdown_frontmatter(tf):
            fixed_count += 1
            print(f"  └─ Fixed: {tf.relative_to(ROOT)}")
            
    print(f"\n✅ Total specification files aligned: {fixed_count}/{len(target_files)}")

if __name__ == "__main__":
    main()

# Metadata: [fix_agent_frontmatter]
