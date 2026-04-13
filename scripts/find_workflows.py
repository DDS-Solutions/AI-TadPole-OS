import json
import os

def find_all_workflows(file_path):
    with open(file_path, "r", encoding="utf-8") as f:
        agents = json.load(f)
    
    workflows = set()
    for agent in agents:
        agent_workflows = agent.get("workflows", [])
        if agent_workflows:
            for wf in agent_workflows:
                if isinstance(wf, str):
                    workflows.add(wf)
    
    print("--- Workflows found in agents.json ---")
    for wf in sorted(list(workflows)):
        print(f"- {wf}")

if __name__ == "__main__":
    find_all_workflows("data/agents.json")
