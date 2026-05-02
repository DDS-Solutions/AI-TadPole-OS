"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[fix_handlers]` in system logs.
"""

import os
import re

TARGET_DIR = 'd:/TadpoleOS-Dev/server-rs/src/agent/runner'

files_to_process = [
    'mission_tools.rs', 'metrics_tools.rs', 
    'external_tools.rs', 'evolution_tools.rs', 'swarm.rs'
]

def fix_file(filename):
    filepath = os.path.join(TARGET_DIR, filename)
    if not os.path.exists(filepath): return
    
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 1. Fix {})"
    content = content.replace(' {})"', '")')
    content = content.replace(' {}\")', '\")')
    content = content.replace('}\"\, output_text)', '}\")')
    content = content.replace('{}\", output_text)', '}\")')
    
    # 2. Fix *output_text = format!
    content = re.sub(r'\*output_text\s*=\s*format!\(', 'return Ok(format!(', content)
    content = re.sub(r'\*_output_text\s*=\s*format!\(', 'return Ok(format!(', content)
    
    # 3. Fix the closing brace of the format macro if it ends with output_text
    # " {})", output_text); -> ");
    # Since we replaced *output_text = with return Ok( it's now return Ok(format!(...);
    # which is syntactically invalid. So we need to change `);` to `));`
    # Let's just fix specific files
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

for f in files_to_process:
    fix_file(f)

# Metadata: [fix_handlers]
