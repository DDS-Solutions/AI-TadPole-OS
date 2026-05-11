"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**INSERT INTO agents ({col_names}) VALUES ({placeholders})**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[restore_agents]` in system logs.
"""

import sqlite3
import json
import os
import sys
from typing import List, Dict, Any, Optional

def print_result(check, status, message):
    icon = "[OK]" if status else "[FAIL]"
    print(f"{icon} [{check}] {message}")

def restore_agents(json_path, db_path):
    if not os.path.exists(json_path):
        print(f"Error: JSON file not found at {json_path}")
        return
    
    print(f"Reading agents from {json_path}...")
    with open(json_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    if isinstance(data, list):
        agents = data
    else:
        agents = [data]
    
    print(f"Found {len(agents)} agents in JSON.")
    
    if not os.path.exists(db_path):
        print(f"Warning: Database not found at {db_path}. It will be created if the directory exists.")
    
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Ensure table exists (Safety)
    cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='agents';")
    if not cursor.fetchone():
        print("Error: 'agents' table does not exist in the database. Please run the server once to initialize schema.")
        conn.close()
        return

    count = 0
    for agent in agents:
        agent_id = agent.get('id')
        if not agent_id: continue
        
        name = agent.get('name', 'Unknown Agent')
        role = agent.get('role', 'General Intelligence Node')
        department = agent.get('department', 'Swarm Core')
        description = agent.get('description', '')
        status = agent.get('status', 'idle')
        
        # model_config mapping
        model_config = agent.get('model_config', agent.get('modelConfig', {}))
        model_id = agent.get('model', agent.get('model_id', model_config.get('model_id', model_config.get('modelId'))))
        provider = model_config.get('provider', 'gemini')
        if not provider or provider == "": provider = "gemini"
        
        api_key = model_config.get('api_key', model_config.get('apiKey'))
        base_url = model_config.get('base_url', model_config.get('baseUrl'))
        system_prompt = model_config.get('system_prompt', model_config.get('systemPrompt'))
        temperature = model_config.get('temperature')
        
        # JSON blobs
        metadata = json.dumps(agent.get('metadata', {}))
        skills = json.dumps(agent.get('skills', []))
        workflows = json.dumps(agent.get('workflows', []))
        mcp_tools = json.dumps(agent.get('mcp_tools', []))
        
        # Additional fields
        theme_color = agent.get('theme_color', agent.get('themeColor'))
        budget_usd = agent.get('budget_usd', agent.get('budgetUsd', 10.0))
        cost_usd = agent.get('cost_usd', agent.get('costUsd', 0.0))
        voice_id = agent.get('voice_id', agent.get('voiceId'))
        voice_engine = agent.get('voice_engine', agent.get('voiceEngine'))
        category = agent.get('category', 'user')
        requires_oversight = 1 if agent.get('requires_oversight') else 0
        working_memory = json.dumps(agent.get('working_memory', {}))
        
        # tokens
        tokens_used = agent.get('tokens_used', agent.get('tokensUsed', 0))
        failure_count = agent.get('failure_count', agent.get('failureCount', 0))
        
        # Model 2/3
        model_2 = agent.get('model_2')
        model_3 = agent.get('model_3')
        model_config2 = json.dumps(agent.get('model_config_2', agent.get('modelConfig2'))) if (agent.get('model_config_2') or agent.get('modelConfig2')) else None
        model_config3 = json.dumps(agent.get('model_config_3', agent.get('modelConfig3'))) if (agent.get('model_config_3') or agent.get('modelConfig3')) else None
        
        # Determine columns dynamically to avoid errors on schema mismatch
        cursor.execute("PRAGMA table_info(agents)")
        db_cols = [row[1] for row in cursor.fetchall()]
        
        fields = {
            "id": agent_id,
            "name": name,
            "role": role,
            "department": department,
            "description": description,
            "model_id": model_id,
            "status": status,
            "theme_color": theme_color,
            "budget_usd": budget_usd,
            "cost_usd": cost_usd,
            "metadata": metadata,
            "skills": skills,
            "workflows": workflows,
            "mcp_tools": mcp_tools,
            "model_2": model_2,
            "model_3": model_3,
            "model_config2": model_config2,
            "model_config3": model_config3,
            "voice_id": voice_id,
            "voice_engine": voice_engine,
            "provider": provider,
            "api_key": api_key,
            "base_url": base_url,
            "system_prompt": system_prompt,
            "temperature": temperature,
            "category": category,
            "requires_oversight": requires_oversight,
            "working_memory": working_memory,
            "tokens_used": tokens_used,
            "failure_count": failure_count
        }
        
        # Only use fields that exist in the DB
        active_fields = {k: v for k, v in fields.items() if k in db_cols}
        col_names = ", ".join(active_fields.keys())
        placeholders = ", ".join(["?"] * len(active_fields))
        update_set = ", ".join([f"{k} = excluded.{k}" for k in active_fields.keys() if k != "id"])
        
        query = f"""
        INSERT INTO agents ({col_names}) VALUES ({placeholders})
        ON CONFLICT(id) DO UPDATE SET {update_set}
        """
        
        cursor.execute(query, list(active_fields.values()))
        count += 1
        
    conn.commit()
    conn.close()
    print_result("AGENT-RESTORE", True, f"Successfully restored {count} agents to {db_path}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python restore_agents.py <source_json> <dest_db>")
        sys.exit(1)
    
    source_json = sys.argv[1]
    dest_db = sys.argv[2]
    restore_agents(source_json, dest_db)

# Metadata: [restore_agents]
