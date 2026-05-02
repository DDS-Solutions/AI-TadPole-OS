"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Runs the master verification suite and returns a high-fidelity summary.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[self_audit_tool]` in system logs.
"""

import json
import subprocess
import os
import sys
import io
from pathlib import Path

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def run_self_audit():
    """Runs the master verification suite and returns a high-fidelity summary."""
    project_path = os.getcwd()
    
    # We use a default localhost URL for the metrics checks
    url = "http://localhost:8000"
    
    print(f"🛠️ [Integrity] Initiating Sovereign Self-Audit of {project_path}...")
    
    cmd = [
        "python", 
        "execution/verify_all.py", 
        project_path, 
        "--url", url,
        "--stop-on-fail"
    ]
    
    try:
        process = subprocess.Popen(
            cmd, 
            stdout=subprocess.PIPE, 
            stderr=subprocess.PIPE,
            text=True,
            encoding='utf-8',
            errors='replace'
        )
        stdout, stderr = process.communicate(timeout=300)
        
        passed = process.returncode == 0
        
        # Extract the final report summary if possible
        summary = "Audit Failed: See details below."
        if "📊 FULL VERIFICATION REPORT" in stdout:
            summary = stdout.split("📊 FULL VERIFICATION REPORT")[-1].strip()
            # Truncate if too long for agent context
            summary = summary[:2000]
            
        report = {
            "status": "PASS" if passed else "FAIL",
            "summary": summary,
            "error_log": stderr[:1000] if not passed else ""
        }
        
        print(json.dumps(report, indent=2))
        return passed

    except Exception as e:
        print(json.dumps({
            "status": "ERROR",
            "summary": f"Audit execution failed: {str(e)}"
        }))
        return False

if __name__ == "__main__":
    success = run_self_audit()
    sys.exit(0 if success else 1)

# Metadata: [self_audit_tool]
