import requests
import json

token = "tadpole-dev-token-2026"
headers = {
    "Authorization": f"Bearer {token}",
    "Content-Type": "application/json"
}

agents = [
    {
        "id": "aegis",
        "name": "Aegis",
        "role": "Security & Intelligence Lead",
        "department": "Security",
        "description": "Swarm oversight and mission intelligence retrieval.",
        "model": "qwen3.5:9b",
        "modelConfig": {
            "provider": "ollama",
            "modelId": "qwen3.5:9b"
        },
        "activeModelSlot": 1,
        "status": "idle",
        "tokensUsed": 0,
        "tokenUsage": {"inputTokens": 0, "outputTokens": 0, "totalTokens": 0},
        "budgetUsd": 50.0,
        "costUsd": 0.0,
        "skills": [],
        "workflows": ["github_scout"],
        "themeColor": "#3b82f6"
    },
    {
        "id": "seeker",
        "name": "Seeker",
        "role": "Specialist",
        "department": "Intelligence",
        "description": "Deep scans codebases and synthesizes technical reports.",
        "model": "llama-3.3-70b-versatile",
        "modelConfig": {
            "provider": "groq",
            "modelId": "llama-3.3-70b-versatile"
        },
        "status": "idle",
        "tokensUsed": 0,
        "tokenUsage": {"inputTokens": 0, "outputTokens": 0, "totalTokens": 0},
        "budgetUsd": 25.0,
        "costUsd": 0.0,
        "skills": [],
        "workflows": ["github_scout"],
        "themeColor": "#10b981"
    }
]

for agent in agents:
    print(f"Registering {agent['name']}...")
    resp = requests.post("http://localhost:8000/v1/agents", headers=headers, json=agent)
    print(f"Status: {resp.status_code}")
    print(f"Response: {resp.text}")
