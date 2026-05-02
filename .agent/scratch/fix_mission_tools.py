"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[fix_mission_tools]` in system logs.
"""

import os
import re

filepath = 'd:/TadpoleOS-Dev/server-rs/src/agent/runner/mission_tools.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix parameter `_)`
content = content.replace('_) -> Result<String, ToolExecutionError>', '_usage: &mut Option<crate::agent::types::TokenUsage>) -> Result<String, ToolExecutionError>')

# Fix format!("{}{}", echo)
content = content.replace('return Ok(format!("{}{}"), echo)', 'return Ok(echo)')
content = content.replace('return Ok(format!("{}{}", echo));', 'return Ok(echo);')

# Fix (Mission completion REJECTED) {}
content = content.replace('return Ok(format!("(Mission completion REJECTED) {}"));', 'return Ok("(Mission completion REJECTED)".to_string());')

# Fix (Codebase read REJECTED by Oversight) {}
content = content.replace('return Ok(format!("(Codebase read REJECTED by Oversight) {}"));', 'return Ok("(Codebase read REJECTED by Oversight)".to_string());')

# Fix `*_output_text = format!("... {}", _output_text);` in handle_pin_mission
content = re.sub(r'\*_output_text\s*=\s*format!\("(.*?) \{\}",\s*_output_text\);', r'return Ok("\1".to_string());', content)

# Fix output_text missing
content = re.sub(r'\*output_text\s*=\s*format!\(', r'return Ok(format!(', content)
content = content.replace('{})", output_text', '}")')
content = content.replace('{})", _output_text', '}")')
content = content.replace('{} ", output_text', '")')
content = content.replace('{}, output_text', ')')
content = content.replace('query, hint, output_text', 'query, hint')
content = content.replace('query, results_text, output_text', 'query, results_text')

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)

# Metadata: [fix_mission_tools]
