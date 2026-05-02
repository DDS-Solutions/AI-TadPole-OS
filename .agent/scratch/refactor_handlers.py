"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[refactor_handlers]` in system logs.
"""

import os
import re

TARGET_DIR = 'd:/TadpoleOS-Dev/server-rs/src/agent/runner'

files_to_process = [
    'swarm.rs', 'mission_tools.rs', 'metrics_tools.rs', 
    'external_tools.rs', 'evolution_tools.rs'
]

# Regex 1: Remove `output_text: &mut String,` from parameters
sig_pattern = re.compile(r'output_text:\s*&mut\s*String,?\s*')
# Regex 2: Change `Result<(), AppError>` to `Result<String, crate::agent::runner::tools::error::ToolExecutionError>`
ret_pattern = re.compile(r'->\s*Result<\(\),\s*AppError>')
# Regex 3: Change `*output_text = format!(...);` and `Ok(())` to `Ok(format!(...))`
# We will do this via a simpler replacement: Just change `*output_text = ` to `let out_tmp = `
# and change `Ok(())` to `Ok(out_tmp)` - wait, some functions return `Ok(())` early or late.
# It's better to just manually edit the files or use sophisticated text replacement.

def process_file(filename):
    filepath = os.path.join(TARGET_DIR, filename)
    if not os.path.exists(filepath): return
    
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
        
    content = content.replace("use crate::error::AppError;", "use crate::error::AppError;\nuse crate::agent::runner::tools::error::ToolExecutionError;")
    content = sig_pattern.sub('', content)
    content = ret_pattern.sub('-> Result<String, ToolExecutionError>', content)
    
    # Simple regex for *output_text = format!("...")
    # This is tricky because the format strings often include `output_text` at the end
    # e.g. *output_text = format!("... {}", output_text)
    # Let's replace ` {}"\, output_text` with `"`
    content = content.replace(', output_text)', ')')
    content = content.replace(' {}", output_text)', '")')
    content = content.replace(' {}\", output_text)', '")')
    
    # Replace `*output_text = ` with `return Ok(`
    content = re.sub(r'\*output_text\s*=\s*(.+?);', r'return Ok(\1);', content)
    
    # Now any remaining `Ok(())` can just be `Ok(String::new())` (should be unreachable if we returned earlier)
    content = content.replace('Ok(())', 'Ok("".to_string())')
    
    # Also fix some issues with specific traits
    content = content.replace('use crate::agent::runner::tools::error::ToolExecutionError;\nuse crate::agent::runner::tools::error::ToolExecutionError;', 'use crate::agent::runner::tools::error::ToolExecutionError;')

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)
        
for f in files_to_process:
    process_file(f)

# Metadata: [refactor_handlers]
