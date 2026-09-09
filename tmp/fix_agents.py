"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[fix_agents]` in system logs.
"""

import re
import os

filepath = 'src/data/mockAgents.ts'
if not os.path.exists(filepath):
    print("File not found")
    exit(1)

content = open(filepath, 'r', encoding='utf-8').read()

# Pattern to find agent objects and check for category
# We look for objects starting with { and containing id: '...' but NOT containing category:
# Since agent objects are multiline, we can use a more robust regex or just iterate.

lines = content.split('\n')
new_lines = []
for line in lines:
    # Match id: '1', id: '2', etc. but only if not followed by category in the same line or nearby.
    if "id: '" in line and "category:" not in line:
        # Check if it's an agent object start or mid
        # Example: { id: '1', name: ... } or just id: '1',
        line = line.replace("id: '", "category: 'ai', id: '")
    new_lines.append(line)

with open(filepath, 'w', encoding='utf-8') as f:
    f.write('\n'.join(new_lines))

print("Transformation complete.")

# Metadata: [fix_agents]
