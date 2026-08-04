#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Subagent Schema Validator (subagent_schema_validator.py)**: Validates subagent handoff 
JSON payloads against formal JSON Schemas stored in `.agent/schemas/`. Ensures zero structural 
drift and guarantees that task handoffs contain required verification steps and input artifacts.

### 🔍 Debugging & Observability
- **Trace Scope**: `execution::subagent_schema_validator`
- **Schema Directory**: `.agent/schemas/`
"""

import sys
import os
import json
from pathlib import Path

# Add execution dir to import path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from py_utils import init_utf8, print_ok, print_warn, print_err, print_step, print_header
from event_logger import log_event

init_utf8()

WORKSPACE_ROOT = Path(__file__).parent.parent
SCHEMAS_DIR = WORKSPACE_ROOT / ".agent" / "schemas"


def load_schema(schema_name: str) -> dict:
    """Load a JSON schema from .agent/schemas/<schema_name>.json."""
    if not schema_name.endswith(".json"):
        schema_name += ".json"
        
    schema_path = SCHEMAS_DIR / schema_name
    if not schema_path.exists():
        raise FileNotFoundError(f"Schema not found: {schema_path}")
        
    with open(schema_path, "r", encoding="utf-8") as f:
        return json.load(f)


def validate_payload(payload: dict, schema_name: str = "directive_schema.json") -> tuple[bool, list[str]]:
    """
    Validate a dictionary payload against a named schema.
    
    :param payload: Target payload dictionary.
    :param schema_name: Name of schema file in .agent/schemas/.
    :return: (is_valid, list_of_error_strings)
    """
    errors = []
    try:
        schema = load_schema(schema_name)
        required_fields = schema.get("required", [])
        
        # Check required fields
        for field in required_fields:
            if field not in payload:
                errors.append(f"Missing required field: '{field}'")
                
        # Validate properties if present
        properties = schema.get("properties", {})
        for key, val in payload.items():
            if key in properties:
                spec = properties[key]
                enum_vals = spec.get("enum")
                if enum_vals and val not in enum_vals:
                    errors.append(f"Field '{key}' value '{val}' not in allowed enum: {enum_vals}")
                min_len = spec.get("minLength")
                if min_len and isinstance(val, str) and len(val) < min_len:
                    errors.append(f"Field '{key}' string length {len(val)} < minLength {min_len}")
                    
        is_valid = len(errors) == 0
        log_event(
            event_type="SCHEMA_VALIDATION",
            action=f"Validated payload against {schema_name}",
            observation={"is_valid": is_valid, "error_count": len(errors), "errors": errors},
            status="COMPLETED" if is_valid else "FAILED"
        )
        return is_valid, errors
        
    except Exception as e:
        err_msg = f"Schema validation exception: {str(e)}"
        log_event("SCHEMA_VALIDATION", err_msg, status="FAILED")
        return False, [err_msg]


def run_self_test() -> bool:
    """Self-test routine for subagent_schema_validator.py."""
    print_header("Subagent Schema Validator Self-Test")
    
    valid_sample = {
        "agent_role": "NEXUS_ENGINEER",
        "task_goal": "Integrate AST PageRank centrality scoring in graph query binary",
        "input_artifacts": ["server-rs/src/bin/graph_query/query_manager.rs"],
        "expected_outputs": ["reports/intelligence/symbol_graph.json"],
        "verification_steps": ["cargo run --bin graph_query -- export"],
        "risk_level": "MODERATE"
    }
    
    invalid_sample = {
        "agent_role": "INVALID_ROLE",
        "task_goal": "Short"
    }
    
    print_step("Validating valid sample directive...")
    is_valid, errs = validate_payload(valid_sample, "directive_schema.json")
    if is_valid:
        print_ok("Valid payload passed schema validation successfully.")
    else:
        print_err(f"Valid payload failed validation: {errs}")
        return False
        
    print_step("Validating invalid sample directive (negative test)...")
    is_valid_neg, errs_neg = validate_payload(invalid_sample, "directive_schema.json")
    if not is_valid_neg:
        print_ok(f"Invalid payload caught correctly with errors: {errs_neg}")
        return True
    else:
        print_err("Invalid payload unexpectedly passed validation!")
        return False


if __name__ == "__main__":
    success = run_self_test()
    sys.exit(0 if success else 1)
