"""
@docs ARCHITECTURE:Infrastructure:Migration

### AI Assist Note
**Script: migrate_industry_templates.py**
Normalizes industry-specific agent templates to the latest snake_case schema.

### 🔍 Debugging & Observability
- **Failure Path**: JSON corruption, invalid path, or missing schema keys.
- **Telemetry Link**: Search `[migrate_industry_templates]` in system logs.
"""

import os
import json

def migrate_json_file(path):
    print(f"[migrate_industry_templates] Processing {path}...")
    with open(path, 'r', encoding='utf-8') as f:
        try:
            data = json.load(f)
        except Exception as e:
            print(f"  Error parsing {path}: {e}")
            return

    if not isinstance(data, dict):
        return

    # 1. Handle Agent Schema (Normalizing to snake_case model_config)
    
    # Resolve model_config
    mc = data.get('model_config') or data.get('modelConfig')
    if mc and isinstance(mc, dict):
        # Normalize keys inside config
        if 'modelId' in mc: mc['model_id'] = mc.pop('modelId')
        if 'systemPrompt' in mc: mc['system_prompt'] = mc.pop('systemPrompt')
        data['model_config'] = mc
        if 'modelConfig' in data: del data['modelConfig']

    # Handle top-level legacy fields
    if 'model' in data:
        mid = data.pop('model')
        if 'model_config' not in data: data['model_config'] = {}
        if 'model_id' not in data['model_config']:
            data['model_config']['model_id'] = mid

    if 'primary_model' in data:
        mid = data.pop('primary_model')
        if 'model_config' not in data: data['model_config'] = {}
        if 'model_id' not in data['model_config']:
            data['model_config']['model_id'] = mid

    if 'system_prompt' in data:
        prompt = data.pop('system_prompt')
        if 'model_config' not in data: data['model_config'] = {}
        if 'system_prompt' not in data['model_config']:
            data['model_config']['system_prompt'] = prompt

    # Final polish on model_config
    if 'model_config' in data:
        mc = data['model_config']
        if 'model_id' in mc:
            data['model_id'] = mc['model_id']
        if 'provider' not in mc and 'model_id' in mc:
            mid = mc['model_id']
            if 'openai' in mid: mc['provider'] = 'openai'
            elif 'anthropic' in mid: mc['provider'] = 'anthropic'
            elif 'gemini' in mid or 'google' in mid: mc['provider'] = 'google'
            else: mc['provider'] = 'openai'

    # Field renames
    if 'budgetUsd' in data: data['budget_usd'] = data.pop('budgetUsd')
    if 'budget_usd' in data: data['budgetUsd'] = data['budget_usd'] # Keeping both for UI/Engine parity
    
    if 'requiresOversight' in data: data['requires_oversight'] = data.pop('requiresOversight')
    if 'requires_oversight' in data: data['requiresOversight'] = data['requires_oversight']

    # 2. Normalize Tool/Server names recursively
    def normalize_value(val):
        if isinstance(val, str):
            val = val.replace('mcp_github-mcp-server_', 'mcp_github_')
            val = val.replace('mcp_brave-search-server_', 'mcp_brave_')
            val = val.replace('github-mcp-server', 'github')
            return val
        if isinstance(val, list):
            return [normalize_value(item) for item in val]
        if isinstance(val, dict):
            return {k: normalize_value(v) for k, v in val.items()}
        return val

    data = normalize_value(data)

    with open(path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2)

root_dir = r"C:\Users\Home Office_PC\.gemini\antigravity\playground\tadpole-os-industry-templates"
for root, dirs, files in os.walk(root_dir):
    if "node_modules" in root: continue
    for file in files:
        if file.endswith(".json"):
            migrate_json_file(os.path.join(root, file))

# Metadata: [migrate_industry_templates]
