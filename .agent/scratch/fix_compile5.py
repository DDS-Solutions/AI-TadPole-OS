"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[fix_compile5]` in system logs.
"""

import os

def fix_mission_tools():
    filepath = 'd:/TadpoleOS-Dev/server-rs/src/agent/runner/mission_tools.rs'
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    content = content.replace("'. {}') {}\"", "'. {}')\"")
    content = content.replace("'):\\n{}\\n{}\"", "'):\\n{}\"")

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

def fix_runner_import(filename):
    filepath = 'd:/TadpoleOS-Dev/server-rs/src/agent/runner/tools/' + filename
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    content = content.replace('use crate::agent::AgentRunner;', 'use crate::agent::runner::AgentRunner;')

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

fix_mission_tools()
fix_runner_import('security.rs')
fix_runner_import('registry.rs')
fix_runner_import('dispatcher.rs')

# Metadata: [fix_compile5]
