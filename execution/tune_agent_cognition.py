"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**tune_agent_cognition**: Autonomous hyperparameter management tool.
Enables the Alpha Agent to modify specialist sampling parameters (Temperature, Top_P, Top_K)
and Reasoning Depth via the TadpoleOS backend API.

### 🔍 Debugging & Observability
- **Failure Path**: 401 Unauthorized (Invalid NEURAL_TOKEN), 404 Not Found (Invalid Agent ID).
- **Telemetry Link**: Search `[CognitionTuning]` in system logs.
"""

import sys
import json
import os
import logging
import requests
from urllib.parse import quote
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry
from typing import Dict, Any, Optional

# --- Configuration & Observability ---
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] [CognitionTuning] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger("tune_agent")

def get_session(retries=3, backoff_factor=0.3, status_forcelist=(500, 502, 504)):
    """Creates a resilient requests session with retry logic."""
    session = requests.Session()
    retry = Retry(
        total=retries,
        read=retries,
        connect=retries,
        backoff_factor=backoff_factor,
        status_forcelist=status_forcelist,
    )
    adapter = HTTPAdapter(max_retries=retry)
    session.mount('http://', adapter)
    session.mount('https://', adapter)
    return session

def validate_range(val: float, min_val: float, max_val: float, name: str):
    """Ensures a value is within the specified range."""
    if val < min_val or val > max_val:
        raise ValueError(f"Parameter '{name}' ({val}) is out of range [{min_val}, {max_val}]")

def tune_agent():
    # 1. Parse & Validate Input
    try:
        # Limit read to 1MB to prevent resource exhaustion
        raw_input = sys.stdin.read(1024 * 1024)
        if not raw_input:
            logger.error("No input received on stdin")
            sys.exit(1)
        input_data = json.loads(raw_input)
    except Exception as e:
        logger.error(f"Failed to parse stdin JSON: {str(e)}")
        sys.exit(1)

    agent_id = input_data.get("agent_id")
    rationale = input_data.get("rationale", "Autonomous hyperparameter optimization.")

    if not agent_id:
        logger.error("Missing required parameter: agent_id")
        sys.exit(1)

    # 2. Environment Configuration
    # [Nexus Note]: Enforcing environment variable resolution.
    token = os.getenv("NEURAL_TOKEN")
    if not token:
        logger.error("Security Violation: NEURAL_TOKEN not found in environment. Aborting.")
        sys.exit(1)
        
    port = os.getenv("PORT", "8000")
    base_url = f"http://127.0.0.1:{port}/v1"
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
        "User-Agent": "TadpoleOS-CognitionTuner/1.0"
    }

    session = get_session()
    logger.info(f"Targeting Agent: {agent_id}")
    logger.info(f"Rationale: {rationale}")

    # 3. Discovery & State Verification
    # [Architectural Note]: Utilizing O(1) direct fetch. Fallback to scan only on 404 for legacy support.
    try:
        safe_id = quote(str(agent_id))
        resp = session.get(f"{base_url}/agents/{safe_id}", headers=headers, timeout=10)
        
        if resp.status_code == 200:
            target_agent = resp.json()
        elif resp.status_code == 404:
            logger.warning(f"Direct fetch failed for '{agent_id}'. Attempting legacy registry scan...")
            # Fallback to list scan (using correct 'data' field per RFC 9457 parity)
            list_resp = session.get(f"{base_url}/agents", headers=headers, timeout=10)
            list_resp.raise_for_status()
            agents_data = list_resp.json()
            items = agents_data.get("data", []) 
            target_agent = next((a for a in items if a.get("id") == agent_id), None)
            
            if not target_agent:
                logger.error(f"Agent '{agent_id}' not found in swarm registry.")
                available = [f"{a.get('id')} ({a.get('role')})" for a in items[:5]]
                logger.info(f"Available agents (first 5): {', '.join(available)}")
                sys.exit(1)
        else:
            resp.raise_for_status()
            target_agent = None
            
    except requests.exceptions.RequestException as e:
        logger.error(f"Registry Discovery Failed: {str(e)}")
        sys.exit(1)

    # 4. Construct Update Payload with Strict Validation
    update_payload: Dict[str, Any] = {}
    
    try:
        if (temp := input_data.get("temperature")) is not None:
            val = float(temp)
            validate_range(val, 0.0, 2.0, "temperature")
            update_payload["temperature"] = val
        
        if (p := input_data.get("top_p")) is not None:
            val = float(p)
            validate_range(val, 0.0, 1.0, "top_p")
            update_payload["topP"] = val
        
        if (k := input_data.get("top_k")) is not None:
            val = int(k)
            validate_range(val, 1, 100, "top_k")
            update_payload["topK"] = val
        
        if (depth := input_data.get("reasoning_depth")) is not None:
            val = int(depth)
            validate_range(val, 1, 64, "reasoning_depth")
            update_payload["reasoningDepth"] = val
            
    except (ValueError, TypeError) as e:
        logger.error(f"Parameter Validation Failed: {str(e)}")
        sys.exit(1)

    if not update_payload:
        logger.warning("No tuning parameters provided. Exiting.")
        return

    # 5. Atomic Update Commitment
    # [Security Note]: Sanitize ID for URL path to prevent traversal.
    safe_id = quote(str(agent_id))
    
    try:
        put_resp = session.put(
            f"{base_url}/agents/{safe_id}",
            headers=headers,
            json=update_payload,
            timeout=10
        )
        put_resp.raise_for_status()
        
        logger.info(f"Successfully tuned Agent {agent_id}")
        print(json.dumps({
            "status": "success",
            "agent_id": agent_id,
            "updates": update_payload,
            "rationale": rationale
        }))
        
    except requests.exceptions.RequestException as e:
        logger.error(f"Persistence Failure: {str(e)}")
        if hasattr(e.response, 'text'):
            logger.debug(f"Backend Error Detail: {e.response.text}")
        sys.exit(1)

if __name__ == "__main__":
    tune_agent()

# Metadata: [tune_agent_cognition]

# Metadata: [tune_agent_cognition]
