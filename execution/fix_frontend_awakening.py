import os

TARGET_DIRS = [
    'src/components',
    'src/hooks',
    'src/layouts',
    'src/pages',
    'src/services',
    'src/stores'
]

REQUIRED_MARKERS = [
    '* @docs ARCHITECTURE:',
    '### AI Assist Note',
    '### 🔍 Debugging & Observability'
]

PILLAR_MAP = {
    'src/components': 'Components',
    'src/hooks': 'Hooks',
    'src/layouts': 'Layouts',
    'src/pages': 'Pages',
    'src/services': 'Services',
    'src/stores': 'Stores'
}

def get_pillar(file_path):
    if '.test.' in file_path or '__tests__' in file_path:
        return 'TestSuites'
    for k, v in PILLAR_MAP.items():
        if k in file_path:
            return v
    return 'Core'

def fix_file(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    missing = [m for m in REQUIRED_MARKERS if m not in content]
    if not missing:
        return False
    
    pillar = get_pillar(file_path)
    is_test = pillar == 'TestSuites'
    
    header = f"""/**
 * @docs ARCHITECTURE:{pillar}
 * 
 * ### AI Assist Note
 * **{'Verification suite for system integrity' if is_test else 'Core functional element for the Tadpole OS engine'}.**
 * 
 * ### 🔍 Debugging & Observability
 * - **{'Failure Path: Test regression or environment mismatch' if is_test else 'Failure Path: Runtime logic error or state corruption'}.**
 * - **Telemetry Link**: Search `[{os.path.basename(file_path)}]` in tracing logs.
 */

"""
    # Prepend header
    new_content = header + content
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(new_content)
    return True

def main():
    fixed_count = 0
    for target in TARGET_DIRS:
        if not os.path.exists(target):
            continue
        for root, _, files in os.walk(target):
            for file in files:
                if file.endswith('.ts') or file.endswith('.tsx'):
                    file_path = os.path.join(root, file)
                    if fix_file(file_path):
                        print(f"Fixed: {file_path}")
                        fixed_count += 1
    print(f"\nTotal Files Fixed: {fixed_count}")

if __name__ == "__main__":
    main()
