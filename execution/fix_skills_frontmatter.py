"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / fix_skills_frontmatter
- **Primary Entrypoints**: `align_skill_frontmatter`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[fix_skills_frontmatter]`
- **Witness Tests**: none declared
"""

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
SKILLS_DIR = ROOT / ".agent" / "skills"

def align_skill_frontmatter(skill_path: Path) -> bool:
    content = skill_path.read_text(encoding='utf-8', errors='ignore')
    
    # Check if existing frontmatter starts at line 1 and is valid
    if content.startswith("---"):
        fm_match = re.match(r'^---\s*\n(.*?)\n---', content, re.DOTALL)
        if fm_match:
            try:
                fm_data = yaml.safe_load(fm_match.group(1))
                if isinstance(fm_data, dict) and "name" in fm_data and "description" in fm_data:
                    return False # Already valid
            except Exception:
                pass

    # Extract skill folder name as default name
    skill_folder_name = skill_path.parent.name
    if skill_folder_name in ("2d-games", "3d-games", "game-art", "game-audio", "game-design", "mobile-games", "multiplayer", "pc-games", "vr-ar", "web-games"):
        skill_name = f"game-dev-{skill_folder_name}"
    elif skill_folder_name == "templates":
        skill_name = "app-builder-templates"
    else:
        skill_name = skill_folder_name

    # Check if a frontmatter block exists anywhere in the file
    fm_match = re.search(r'---\s*\n(name:\s*.*?\n---)', content, re.DOTALL)
    if fm_match:
        fm_block = fm_match.group(1).strip()
        pre_fm = content[:fm_match.start()].strip()
        post_fm = content[fm_match.end():].strip()
        
        # Clean header
        new_content = f"---\n{fm_block}\n\n"
        if pre_fm:
            new_content += f"{pre_fm}\n\n"
        if post_fm:
            new_content += f"{post_fm}\n"
    else:
        # Frontmatter missing entirely: synthesize valid L1 frontmatter block
        # Extract title from markdown if available
        title_match = re.search(r'#\s+(.+)', content)
        title = title_match.group(1).strip() if title_match else skill_name.replace('-', ' ').title()
        
        desc = f"Expert guidelines, principles, and procedures for {title}."
        
        # Strip any invalid top marker
        clean_content = content.strip()
        
        new_content = f"---\nname: {skill_name}\ndescription: \"{desc}\"\n---\n\n{clean_content}\n"
        
    skill_path.write_text(new_content, encoding='utf-8')
    return True

def main():
    print(f"🔧 [fix_skills_frontmatter] Aligning frontmatter across skills in: {SKILLS_DIR}...")
    skill_files = list(SKILLS_DIR.glob("**/SKILL.md"))
    
    fixed_count = 0
    for sf in skill_files:
        if align_skill_frontmatter(sf):
            fixed_count += 1
            print(f"  └─ Fixed: {sf.relative_to(ROOT)}")
            
    print(f"\n✅ Total SKILL.md files aligned: {fixed_count}/{len(skill_files)}")

if __name__ == "__main__":
    main()
