import os
import re

def replace_in_file(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Case-sensitive mapping
    replacements = {
        r'\bcapabilities\b': 'skills',
        r'\bCapabilities\b': 'Skills',
        r'\bCAPABILITIES\b': 'SKILLS',
        r'\bcapability\b': 'skill',
        r'\bCapability\b': 'Skill',
        r'\bCAPABILITY\b': 'SKILL'
    }
    
    new_content = content
    for pattern, replacement in replacements.items():
        new_content = re.sub(pattern, replacement, new_content)
    
    if new_content != content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"Updated: {file_path}")

def main():
    target_dirs = ['docs', 'server-rs/src', 'src', 'execution']
    target_files = ['README.md', 'task.md', 'openapi.yaml']
    
    for d in target_dirs:
        if not os.path.exists(d): continue
        for root, _, files in os.walk(d):
            for file in files:
                if file.endswith(('.md', '.json', '.yaml', '.rs', '.ts', '.tsx')):
                    replace_in_file(os.path.join(root, file))
    
    for f in target_files:
        if os.path.exists(f):
            replace_in_file(f)

if __name__ == "__main__":
    main()
