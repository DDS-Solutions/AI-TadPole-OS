"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[fix_mission_tools2]` in system logs.
"""

import os

filepath = 'd:/TadpoleOS-Dev/server-rs/src/agent/runner/mission_tools.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix 1: 165
# return Ok(format!( ... \n            })
content = content.replace(
'''            return Ok(format!(
                "(Mission '{}' marked as COMPLETED by {}.)",
                ctx.mission_id, ctx.name
            )''',
'''            return Ok(format!(
                "(Mission '{}' marked as COMPLETED by {}.)",
                ctx.mission_id, ctx.name
            ))'''
)

# Fix 2: 205
content = content.replace('cycle.) {}", _output_text);', 'cycle.)");')

# Fix 3 & 4: 273 and 278
content = content.replace(
'''            return Ok(format!(
                "(SEARCH NO RESULTS for '{}' with hint '{}')",
                query, hint
            )''',
'''            return Ok(format!(
                "(SEARCH NO RESULTS for '{}' with hint '{}')",
                query, hint
            ))'''
)

content = content.replace(
'''            return Ok(format!(
                "(SEARCH RESULTS for '{}'):\n{}",
                query, results_text
            )''',
'''            return Ok(format!(
                "(SEARCH RESULTS for '{}'):\n{}",
                query, results_text
            ))'''
)

# Fix 5: 315
content = content.replace(
'''            return Ok(format!(
                "(READ MEMORY {})",
                doc_path
            )''',
'''            return Ok(format!(
                "(READ MEMORY {})",
                doc_path
            ))'''
)

# Fix 6: 422
content = content.replace('"(SECURITY BLOCKED:"), e));', '"(SECURITY BLOCKED: {})", e));')
content = content.replace('"(CODEBASE READ FAILED for {}:"), path_str, e));', '"(CODEBASE READ FAILED for {}: {})", path_str, e));')

# Fix 7: 509
content = content.replace(
'''        return Ok(format!(
            "(ARCHIVED TO LONG-TERM MEMORY at {})",
            timestamp
        )''',
'''        return Ok(format!(
            "(ARCHIVED TO LONG-TERM MEMORY at {})",
            timestamp
        ))'''
)

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)

# Metadata: [fix_mission_tools2]
