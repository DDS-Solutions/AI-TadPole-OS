"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / update_starter_kits

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import json
import glob
import os

agent_files = glob.glob('starter_kits/*/agents/*.json')
print(f"Found {len(agent_files)} starter kit agent files.")

for file_path in agent_files:
    with open(file_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    data['model_id'] = 'stealth/oxalpha'
    data['model_2'] = 'stealth/oxalpha'
    data['model_3'] = 'stealth/oxalpha'
    data['budgetUsd'] = 10.0
    data['budget_usd'] = 10.0
    data['status'] = 'offline'
    
    if 'model_config' in data and isinstance(data['model_config'], dict):
        data['model_config']['model_id'] = 'stealth/oxalpha'
        data['model_config']['provider'] = 'openrouter'
    else:
        data['model_config'] = {
            'model_id': 'stealth/oxalpha',
            'provider': 'openrouter'
        }
        
    data['model_config2'] = {
        'model_id': 'stealth/oxalpha',
        'provider': 'openrouter'
    }
    data['model_config3'] = {
        'model_id': 'stealth/oxalpha',
        'provider': 'openrouter'
    }
    
    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2)
        f.write('\n')
        
    print(f"Updated {file_path}")

print("All starter kit agent files successfully updated.")
