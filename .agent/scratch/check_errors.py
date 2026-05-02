"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[check_errors]` in system logs.
"""

import subprocess
import json
import os

def check_errors():
    cwd = 'd:/TadpoleOS-Dev/server-rs'
    proc = subprocess.Popen(['cargo', 'check', '--message-format=json'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=cwd)
    
    for line in proc.stdout:
        try:
            msg = json.loads(line)
            if msg.get('reason') == 'compiler-message':
                message = msg.get('message', {})
                if message.get('level') == 'error':
                    print(message.get('rendered'))
        except json.JSONDecodeError:
            continue

if __name__ == "__main__":
    check_errors()

# Metadata: [check_errors]
