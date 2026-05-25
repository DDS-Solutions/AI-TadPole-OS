"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[replace_styles]` in system logs.
"""

import os
import re

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original = content
    
    # 1. Cards: bg-zinc-900 border border-zinc-800 rounded-xl (or rounded-lg) p-4 (or p-6)
    # Replaced by sovereign-card
    content = re.sub(r'bg-zinc-900\s+border\s+border-zinc-800\s+rounded-(?:xl|lg|md)\s+p-[2-6]\s+cursor-pointer\s+hover:shadow-lg\s+hover:border-\w+-\d+(?:/\d+)?\s+transition-all', 'sovereign-card cursor-pointer', content)
    content = re.sub(r'bg-zinc-900\s+border\s+border-zinc-800\s+rounded-(?:xl|lg|md)\s+p-[2-6]', 'sovereign-card', content)
    
    # 2. Panels: bg-zinc-950 border border-zinc-900 rounded-xl
    content = re.sub(r'bg-zinc-950/50\s+backdrop-blur-md\s+border\s+border-zinc-900\s+rounded-xl', 'sovereign-panel', content)
    
    # 3. Headers: backdrop-blur-md -> backdrop-blur-xl
    content = re.sub(r'backdrop-blur-md\s+sticky', 'backdrop-blur-xl sticky', content)
    
    # 4. Standalone zinc colors (for inputs, small containers, etc.) -> semantic vars
    # We'll use Tailwind 4 arbitrary vars or custom theme vars.
    # We defined these in index.css as --color-theme-900, but they map to zinc anyway.
    # Actually, TadpoleOS uses standard tailwind classes for zinc. The goal is to use the semantic colors where applicable.
    # bg-zinc-900 -> bg-[color:var(--color-surface)]
    # border-zinc-800 -> border-[color:var(--color-border)]
    # bg-zinc-950 -> bg-[color:var(--color-background)]
    content = content.replace('bg-zinc-950', 'bg-[color:var(--color-background)]')
    content = content.replace('bg-zinc-900', 'bg-[color:var(--color-surface)]')
    content = content.replace('border-zinc-800', 'border-[color:var(--color-border)]')
    content = content.replace('border-zinc-900', 'border-[color:var(--color-surface)]')
    
    # 5. Fix any double classes that might have been created
    content = content.replace('bg-[color:var(--color-surface)]/80', 'bg-[color:color-mix(in_srgb,var(--color-surface)_80%,transparent)]')
    content = content.replace('bg-[color:var(--color-background)]/80', 'bg-[color:color-mix(in_srgb,var(--color-background)_80%,transparent)]')
    content = content.replace('bg-[color:var(--color-background)]/50', 'bg-[color:color-mix(in_srgb,var(--color-background)_50%,transparent)]')
    
    # Status Badge colors (warning, error, info, success)
    content = re.sub(r'text-green-500 bg-green-500/10', 'text-success-text bg-success-bg', content)
    content = re.sub(r'text-amber-400 bg-amber-500/10', 'text-warning-text bg-warning-bg', content)
    content = re.sub(r'text-amber-500 bg-amber-500/10', 'text-warning-text bg-warning-bg', content)
    content = re.sub(r'text-blue-400 bg-blue-500/10', 'text-info-text bg-info-bg', content)
    content = re.sub(r'text-blue-500 bg-blue-500/10', 'text-info-text bg-info-bg', content)
    content = re.sub(r'text-emerald-400 bg-emerald-500/10', 'text-success-text bg-success-bg', content)
    content = re.sub(r'text-rose-400 bg-rose-500/10', 'text-danger-text bg-danger-bg', content)
    content = re.sub(r'text-red-500 bg-red-500/10', 'text-danger-text bg-danger-bg', content)

    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Updated: {filepath}")

def main():
    src_dir = os.path.join('d:\\', 'TadpoleOS-Dev', 'src')
    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if file.endswith(('.tsx', '.ts')):
                process_file(os.path.join(root, file))

if __name__ == '__main__':
    main()

# Metadata: [replace_styles]
