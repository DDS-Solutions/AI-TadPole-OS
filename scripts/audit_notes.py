"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Developer Scripts / audit_notes
- **Primary Entrypoints**: `scan_file`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import os
import re
import json

def scan_file(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            
        assist_note = re.search(r'### AI Assist Note(.*?)(?:###|$)', content, re.DOTALL)
        debug_obs = re.search(r'### 🔍 Debugging & Observability(.*?)(?:###|$)', content, re.DOTALL)
        
        return {
            'has_assist_note': assist_note is not None,
            'assist_note_content': assist_note.group(1).strip() if assist_note else None,
            'has_debug_obs': debug_obs is not None,
            'debug_obs_content': debug_obs.group(1).strip() if debug_obs else None
        }
    except Exception as e:
        return {'error': str(e)}

def main():
    results = {}
    extensions = ('.ts', '.tsx', '.rs', '.py', '.md')
    exclude_dirs = {'.git', 'node_modules', 'dist', 'target', '.tmp', 'scratch'}
    
    for root, dirs, files in os.walk('.'):
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        
        for file in files:
            if file.endswith(extensions):
                rel_path = os.path.relpath(os.path.join(root, file), '.')
                results[rel_path] = scan_file(rel_path)
                
    with open('docs/AI_ASSIST_NOTES_AUDIT.json', 'w', encoding='utf-8') as f:
        json.dump(results, f, indent=2)
    
    print(f"Scanned {len(results)} files. Report saved to docs/AI_ASSIST_NOTES_AUDIT.json")

if __name__ == "__main__":
    main()
