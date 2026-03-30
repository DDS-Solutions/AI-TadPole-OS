import requests
import json
import time

token = "tadpole-dev-token-2026"
headers = {
    "Authorization": f"Bearer {token}",
    "Content-Type": "application/json"
}

# Mission payload for Aegis
payload = {
    "message": "Research Tadpole OS repository content and share your findings.",
    "clusterId": "intelligence-gathering",
    "department": "Intelligence"
}

print("Launching mission for Aegis...")
resp = requests.post("http://localhost:8000/v1/agents/aegis/tasks", headers=headers, json=payload)
print(f"Status: {resp.status_code}")
print(f"Response: {resp.text}")

if resp.status_code == 201:
    mission_id = resp.json().get("missionId")
    print(f"Mission started: {mission_id}")
    
    # Wait for completion (poll)
    for _ in range(30):
        time.sleep(10)
        status_resp = requests.get(f"http://localhost:8000/v1/missions/{mission_id}", headers=headers)
        if status_resp.status_code == 200:
            status = status_resp.json().get("status")
            print(f"Mission status: {status}")
            if status in ["completed", "failed"]:
                break
        else:
            print(f"Failed to get status: {status_resp.status_code}")
            break
