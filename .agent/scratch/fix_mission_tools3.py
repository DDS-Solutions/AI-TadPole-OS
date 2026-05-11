"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[fix_mission_tools3]` in system logs.
"""

import os

filepath = 'd:/TadpoleOS-Dev/server-rs/src/agent/runner/mission_tools.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix 165
content = content.replace('ctx.mission_id, report\n            );', 'ctx.mission_id, report\n            ));')
# Fix 205
content = content.replace('cycle.)");', 'cycle.)"));')
# Fix 273
content = content.replace('query, hint\n            );', 'query, hint\n            ));')
# Fix 278
content = content.replace('query, results_text\n            );', 'query, results_text\n            ));')
# Fix 315
content = content.replace('path_str\n            );', 'path_str\n            ));')
# Fix 509
content = content.replace('cap_type_str, name, proposal_id\n        );', 'cap_type_str, name, proposal_id\n        ));')
# Fix 580
content = content.replace('return Ok(format!("(No recognizable symbols found in"), path_str));', 'return Ok(format!("(No recognizable symbols found in {})", path_str));')
content = content.replace('return Ok(format!("(LIST SYMBOLS FAILED for {}:"), path_str, e));', 'return Ok(format!("(LIST SYMBOLS FAILED for {}: {})", path_str, e));')

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)

# Metadata: [fix_mission_tools3]
