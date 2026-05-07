"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[fix_mission_tools4]` in system logs.
"""

import os

filepath = 'd:/TadpoleOS-Dev/server-rs/src/agent/runner/mission_tools.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('return Ok(format!("(SYMBOL \'{}\' NOT FOUND in"), symbol_name, path_str));', 'return Ok(format!("(SYMBOL \'{}\' NOT FOUND in {})", symbol_name, path_str));')
content = content.replace('return Ok(format!("(GET SYMBOL FAILED for {}:"), path_str, e));', 'return Ok(format!("(GET SYMBOL FAILED for {}: {})", path_str, e));')

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)

# Metadata: [fix_mission_tools4]
