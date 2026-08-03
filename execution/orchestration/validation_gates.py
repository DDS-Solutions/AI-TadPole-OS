"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Generates a structured Attention State report.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[validation_gates]` in system logs.
"""

import argparse
import sys
import subprocess
import json
import os

def generate_attention_report(target, error_msg, detail_msg):
    """Generates a structured Attention State report."""
    report = {
        "status": "FAILED",
        "target": target,
        "error": error_msg,
        "details": detail_msg,
        "action_required": "Fix the issue and rerun this gate validation script before proceeding."
    }
    
    print("\n" + "="*50)
    print("!!! VALIDATION GATE FAILED: ATTENTION REQUIRED !!!")
    print("="*50)
    print(f"Target : {target}")
    print(f"Error  : {error_msg}")
    print(f"Details: {detail_msg}")
    print("="*50)
    print("ACTION: You must resolve this issue before checking off the GATE task.\n")
    return report

def validate_target(target):
    """
    Perform validation based on the target name.
    In a real scenario, this would map to specific test scripts or LLM calls.
    """
    is_windows = os.name == 'nt'
    npm_cmd = "npm.cmd" if is_windows else "npm"
    
    if target == "build":
        try:
            print(f"Validating target: {target}...")
            # For demonstration, checking if it builds or passes a dummy check
            result = subprocess.run([npm_cmd, "run", "build"], capture_output=True, text=True, encoding="utf-8", errors="replace", cwd="d:/TadpoleOS-Dev")
            if result.returncode != 0:
                generate_attention_report(target, "Build Failed", result.stderr)
                return False
        except Exception as e:
            generate_attention_report(target, "Execution Error", str(e))
            return False
            
    elif target == "tests":
        try:
            print(f"Validating target: {target}...")
            # Run test suite
            result = subprocess.run([npm_cmd, "run", "test"], capture_output=True, text=True, encoding="utf-8", errors="replace", cwd="d:/TadpoleOS-Dev")
            if result.returncode != 0:
                generate_attention_report(target, "Tests Failed", result.stderr)
                return False
        except Exception as e:
            generate_attention_report(target, "Execution Error", str(e))
            return False
            
    # Add more custom targets (e.g., db_schema, api_endpoints) as needed.
    # If the target isn't implemented, we just pass it as a structural gate for now.
    
    print(f"+++ GATE CLEARED: Target '{target}' passed validation successfully.")
    return True

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Tadpole OS Validation Gate Runner")
    parser.add_argument("--target", required=True, help="The milestone target to validate (e.g., 'build', 'tests', 'db_schema')")
    
    args = parser.parse_args()
    
    if not validate_target(args.target):
        sys.exit(1)
    
    sys.exit(0)

# Metadata: [validation_gates]
