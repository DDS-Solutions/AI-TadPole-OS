#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / mcp-builder / mcp_validator
- **Primary Entrypoints**: `validate_tool_schema`, `validate_mcp_config`, `scaffold_mcp_server`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic MCP schema and config validation without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[mcp_validator]`, `[SCHEMA]`, `[CONFIG]`, `[SCAFFOLD]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import re
import json
import argparse
from pathlib import Path
from typing import Dict, List, Any, Optional

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

TOOL_NAME_PATTERN = re.compile(r'^[a-zA-Z0-9_\-]+$')

def validate_tool_schema(tool: Dict[str, Any]) -> Dict[str, Any]:
    """Validate a single tool definition against the Model Context Protocol schema."""
    errors: List[str] = []
    warnings: List[str] = []

    # 1. Name validation
    name = tool.get("name")
    if not name or not isinstance(name, str):
        errors.append("Missing or invalid 'name' attribute (must be non-empty string)")
    elif not TOOL_NAME_PATTERN.match(name):
        errors.append(f"Tool name '{name}' contains invalid characters (only alphanumeric, dash, and underscore allowed)")
    elif len(name) > 64:
        warnings.append(f"Tool name '{name}' is unusually long ({len(name)} chars > 64)")

    # 2. Description validation
    desc = tool.get("description")
    if not desc or not isinstance(desc, str):
        errors.append("Missing or invalid 'description' attribute (must be non-empty string)")
    elif len(desc.strip()) < 10:
        warnings.append("Tool description is very short; provide clear instruction for LLM caller")

    # 3. InputSchema validation
    input_schema = tool.get("inputSchema")
    if input_schema is None:
        errors.append("Missing required 'inputSchema' attribute")
    elif not isinstance(input_schema, dict):
        errors.append("'inputSchema' must be a JSON object dictionary")
    else:
        schema_type = input_schema.get("type")
        if schema_type != "object":
            errors.append(f"'inputSchema.type' must be 'object', got '{schema_type}'")

        properties = input_schema.get("properties")
        if properties is not None and not isinstance(properties, dict):
            errors.append("'inputSchema.properties' must be a JSON object dictionary")
        elif properties:
            for prop_name, prop_spec in properties.items():
                if not isinstance(prop_spec, dict):
                    errors.append(f"Property '{prop_name}' must be an object dictionary")
                    continue
                if "type" not in prop_spec:
                    warnings.append(f"Property '{prop_name}' has no declared 'type'")
                if "description" not in prop_spec:
                    warnings.append(f"Property '{prop_name}' has no declared 'description'")

        required = input_schema.get("required")
        if required is not None:
            if not isinstance(required, list):
                errors.append("'inputSchema.required' must be an array of property names")
            elif properties:
                for req_prop in required:
                    if req_prop not in properties:
                        errors.append(f"Required property '{req_prop}' not defined in 'inputSchema.properties'")

    is_valid = len(errors) == 0
    return {
        "valid": is_valid,
        "name": name or "unknown",
        "errors": errors,
        "warnings": warnings
    }

def validate_mcp_config(config: Dict[str, Any]) -> Dict[str, Any]:
    """Validate an MCP configuration object (e.g. mcp_config.json or mcpServers)."""
    errors: List[str] = []
    warnings: List[str] = []

    servers = config.get("mcpServers") if "mcpServers" in config else config
    if not isinstance(servers, dict) or not servers:
        errors.append("Configuration does not contain a valid 'mcpServers' object dictionary")
        return {"valid": False, "server_count": 0, "errors": errors, "warnings": warnings}

    for server_name, server_def in servers.items():
        if not isinstance(server_def, dict):
            errors.append(f"Server '{server_name}' definition must be an object dictionary")
            continue

        command = server_def.get("command")
        url = server_def.get("url")
        if not command and not url:
            errors.append(f"Server '{server_name}' must define either 'command' (stdio) or 'url' (SSE/HTTP)")

        if command:
            if not isinstance(command, str):
                errors.append(f"Server '{server_name}' 'command' must be a string executable path")
            args = server_def.get("args")
            if args is not None and not isinstance(args, list):
                errors.append(f"Server '{server_name}' 'args' must be a list of strings")

        env = server_def.get("env")
        if env is not None:
            if not isinstance(env, dict):
                errors.append(f"Server '{server_name}' 'env' must be an object dictionary")
            else:
                for k, v in env.items():
                    if any(secret in k.upper() for secret in ['KEY', 'SECRET', 'TOKEN', 'PASSWORD']):
                        if v and not str(v).startswith("${"):
                            warnings.append(f"Server '{server_name}' may contain plaintext secret in env '{k}' (use env substitution)")

    is_valid = len(errors) == 0
    return {
        "valid": is_valid,
        "server_count": len(servers),
        "errors": errors,
        "warnings": warnings
    }

def scaffold_mcp_server(name: str, language: str = "python", output_path: Optional[Path] = None) -> str:
    """Generate a production-grade template for a stdio MCP server."""
    if language.lower() in ["ts", "typescript"]:
        template = f"""import {{ Server }} from "@modelcontextprotocol/sdk/server/index.js";
import {{ StdioServerTransport }} from "@modelcontextprotocol/sdk/server/stdio.js";
import {{ CallToolRequestSchema, ListToolsRequestSchema }} from "@modelcontextprotocol/sdk/types.js";

const server = new Server(
  {{ name: "{name}", version: "1.0.0" }},
  {{ capabilities: {{ tools: {{}} }} }}
);

// Register Tool Definitions
server.setRequestHandler(ListToolsRequestSchema, async () => ({{
  tools: [
    {{
      name: "example_tool",
      description: "An example tool that returns structured data.",
      inputSchema: {{
        type: "object",
        properties: {{
          message: {{ type: "string", description: "Input message to echo." }}
        }},
        required: ["message"]
      }}
    }}
  ]
}}));

// Handle Tool Execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {{
  if (request.params.name === "example_tool") {{
    const message = String(request.params.arguments?.message ?? "");
    // CRITICAL: Always log to console.error (stderr), NEVER console.log (stdout)
    console.error(`[mcp_server] Executing example_tool with message: ${{message}}`);
    return {{
      content: [{{ type: "text", text: `Echo: ${{message}}` }}],
      isError: false
    }};
  }}
  throw new Error(`Tool not found: ${{request.params.name}}`);
}});

async function main() {{
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("[mcp_server] Server running on stdio transport");
}}

main().catch((err) => {{
  console.error("[mcp_server] Fatal error:", err);
  process.exit(1);
}});
"""
    else:
        template = f"""#!/usr/bin/env python3
import sys
import json
import logging

# CRITICAL: Configure logging to stderr ONLY. Writing to stdout corrupts JSON-RPC!
logging.basicConfig(
    stream=sys.stderr,
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s'
)
logger = logging.getLogger("{name}")

TOOLS = [
    {{
        "name": "example_tool",
        "description": "An example tool that returns structured data.",
        "inputSchema": {{
            "type": "object",
            "properties": {{
                "message": {{
                    "type": "string",
                    "description": "Input message to echo."
                }}
            }},
            "required": ["message"]
        }}
    }}
]

def handle_tools_list(request_id):
    return {{
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {{"tools": TOOLS}}
    }}

def handle_tools_call(request_id, params):
    tool_name = params.get("name")
    args = params.get("arguments", {{}})
    logger.info(f"Executing tool: {{tool_name}} with args: {{args}}")

    if tool_name == "example_tool":
        msg = args.get("message", "")
        return {{
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {{
                "content": [{{"type": "text", "text": f"Echo: {{msg}}"}}]
            }}
        }}

    return {{
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {{"code": -32601, "message": f"Method not found: {{tool_name}}"}}
    }}

def main():
    logger.info("Starting stdio MCP server...")
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            req = json.loads(line)
            method = req.get("method")
            req_id = req.get("id")

            if method == "tools/list":
                resp = handle_tools_list(req_id)
            elif method == "tools/call":
                resp = handle_tools_call(req_id, req.get("params", {{}}))
            elif method == "initialize":
                resp = {{
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {{
                        "protocolVersion": "2024-11-05",
                        "serverInfo": {{"name": "{name}", "version": "1.0.0"}},
                        "capabilities": {{"tools": {{}}}}
                    }}
                }}
            else:
                resp = {{
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "error": {{"code": -32601, "message": f"Unsupported method: {{method}}"}}
                }}

            # Output ONLY the JSON-RPC response to stdout followed by newline
            sys.stdout.write(json.dumps(resp) + "\\n")
            sys.stdout.flush()

        except Exception as e:
            logger.error(f"Error handling request: {{e}}")
            err_resp = {{"jsonrpc": "2.0", "id": None, "error": {{"code": -32700, "message": str(e)}}}}
            sys.stdout.write(json.dumps(err_resp) + "\\n")
            sys.stdout.flush()

if __name__ == "__main__":
    main()
"""
    if output_path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(template, encoding='utf-8')

    return template

def run_self_tests() -> bool:
    """Execute comprehensive automated self-tests."""
    print("[mcp_validator] Executing self-test suite...")

    try:
        # Test 1: Valid tool schema
        valid_tool = {
            "name": "fetch_record",
            "description": "Fetch a specific database record by ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Unique record identifier"}
                },
                "required": ["id"]
            }
        }
        res1 = validate_tool_schema(valid_tool)
        assert res1["valid"], f"Valid tool schema failed: {res1['errors']}"
        print("[mcp_validator] [SCHEMA] Valid tool schema passed validation.")

        # Test 2: Invalid tool schema detection
        invalid_tool = {
            "name": "bad tool with spaces!",
            "description": "bad",
            "inputSchema": {
                "type": "string"  # Must be 'object'
            }
        }
        res2 = validate_tool_schema(invalid_tool)
        assert not res2["valid"], "Invalid tool schema was erroneously marked valid"
        assert len(res2["errors"]) >= 2
        print("[mcp_validator] [SCHEMA] Invalid schema & name violations detected correctly.")

        # Test 3: MCP Configuration validation
        valid_config = {
            "mcpServers": {
                "test-server": {
                    "command": "python",
                    "args": ["server.py"]
                }
            }
        }
        res3 = validate_mcp_config(valid_config)
        assert res3["valid"], f"Valid MCP config failed: {res3['errors']}"
        print("[mcp_validator] [CONFIG] Server configuration dictionary validated.")

        # Test 4: Scaffolding test
        scaffold_py = scaffold_mcp_server("test_engine", language="python")
        assert "sys.stderr" in scaffold_py
        assert 'name": "test_engine"' in scaffold_py
        print("[mcp_validator] [SCAFFOLD] Stdio server code generator verified.")

    except Exception as e:
        print(f"[mcp_validator] [FAIL] Self-test failed: {e}")
        return False

    print("[mcp_validator] [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Sovereign Model Context Protocol (MCP) Tool & Configuration Validator."
    )
    parser.add_argument("path", nargs="?", help="Path to tool schema JSON or mcp_config.json file.")
    parser.add_argument("--tool-schema", action="store_true", help="Validate target file as an MCP Tool Schema.")
    parser.add_argument("--config", action="store_true", help="Validate target file as an MCP Server Config.")
    parser.add_argument("--scaffold-py", action="store_true", help="Generate a Python stdio MCP server template.")
    parser.add_argument("--scaffold-ts", action="store_true", help="Generate a TypeScript stdio MCP server template.")
    parser.add_argument("--name", type=str, default="custom_mcp_server", help="Name for scaffolded server.")
    parser.add_argument("--output", type=str, help="Output file path for scaffolded server.")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format.")
    parser.add_argument("--test", action="store_true", help="Run automated self-tests.")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.scaffold_py or args.scaffold_ts:
        lang = "typescript" if args.scaffold_ts else "python"
        out_p = Path(args.output) if args.output else None
        code = scaffold_mcp_server(args.name, language=lang, output_path=out_p)
        if args.json:
            print(json.dumps({"status": "scaffolded", "language": lang, "name": args.name, "output": str(out_p)}))
        else:
            print(f"[mcp_validator] [SCAFFOLD] Generated {lang} MCP server template for '{args.name}'")
            if out_p:
                print(f"Written to: {out_p}")
            else:
                print(code)
        sys.exit(0)

    if not args.path:
        parser.print_help()
        sys.exit(0)

    path = Path(args.path)
    if not path.exists():
        print(f"[mcp_validator] [FAIL] Target path not found: {args.path}")
        sys.exit(1)

    try:
        content = json.loads(path.read_text(encoding='utf-8'))
    except Exception as e:
        print(f"[mcp_validator] [FAIL] Could not parse JSON file: {e}")
        sys.exit(1)

    # Determine validation type
    if args.tool_schema or ("name" in content and "inputSchema" in content):
        res = validate_tool_schema(content)
        val_type = "Tool Schema"
    else:
        res = validate_mcp_config(content)
        val_type = "MCP Config"

    if args.json:
        print(json.dumps(res, indent=2))
    else:
        print(f"[mcp_validator] Auditing {val_type}: {path}")
        print(f"Status: {'VALID' if res['valid'] else 'INVALID'}")
        if res.get("errors"):
            print("Errors:")
            for err in res["errors"]:
                print(f"  ❌ {err}")
        if res.get("warnings"):
            print("Warnings:")
            for warn in res["warnings"]:
                print(f"  ⚠️ {warn}")
        if res["valid"]:
            print("[mcp_validator] [PASS] Validation successful.")
        else:
            print("[mcp_validator] [FAIL] Schema or configuration violations found.")

    sys.exit(0 if res["valid"] else 1)

if __name__ == "__main__":
    main()
