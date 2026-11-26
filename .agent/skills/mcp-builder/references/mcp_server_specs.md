# MCP Server Specifications & Transport Protocols (L3)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mcp-builder / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Malformed JSON-RPC payloads, stdout log contamination, or transport disconnects.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[MCP_BUILDER]`)

---

## 1. Model Context Protocol (MCP) Architecture Overview

The Model Context Protocol (MCP) establishes an open, standard JSON-RPC 2.0 protocol enabling AI agents and LLM host clients (such as the Antigravity IDE, Claude Desktop, and CLI runners) to interact securely with local and remote tools, data resources, and prompt templates.

```
┌──────────────────┐               ┌──────────────────┐
│   Host Client    │  JSON-RPC 2.0 │    MCP Server    │
│  (Antigravity)   │ <===========> │  (Tools / Data)  │
└──────────────────┘  stdio / SSE  └──────────────────┘
```

---

## 2. Standard JSON-RPC 2.0 Protocol Schemas

### 1. Tool Listing (`tools/list`)
The client queries available tools:
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "query_database",
        "description": "Execute a safe read-only SQL query against the SQLite repository.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "query": {
              "type": "string",
              "description": "SQL SELECT query string to execute."
            },
            "limit": {
              "type": "integer",
              "description": "Maximum number of rows to return (default 50).",
              "default": 50
            }
          },
          "required": ["query"]
        }
      }
    ]
  }
}
```

### 2. Tool Execution (`tools/call`)
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "query_database",
    "arguments": {
      "query": "SELECT id, name FROM agents LIMIT 5;"
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "[{\"id\": 1, \"name\": \"Agent of Nine\"}]"
      }
    ],
    "isError": false
  }
}
```

### 3. Resource Reading (`resources/read`)
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "resources/read",
  "params": {
    "uri": "repo://docs/architecture"
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "contents": [
      {
        "uri": "repo://docs/architecture",
        "mimeType": "text/markdown",
        "text": "# Architecture Specification..."
      }
    ]
  }
}
```

---

## 3. Standard JSON-RPC Error Handling

MCP servers must return standardized JSON-RPC error codes rather than crashing or printing unhandled tracebacks:

| Error Code | Meaning | When to Use |
|---|---|---|
| `-32700` | Parse error | Received JSON could not be parsed by the server. |
| `-32600` | Invalid Request | Payload does not conform to JSON-RPC 2.0 structure. |
| `-32601` | Method not found | The requested method does not exist on the server. |
| `-32602` | Invalid params | Tool arguments failed JSON Schema validation. |
| `-32603` | Internal error | Unhandled internal exception occurred during tool execution. |

### Example Error Response:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32602,
    "message": "Invalid params: Missing required field 'query'"
  }
}
```

---

## 4. Transport Protocols & Non-Negotiables

### 1. `stdio` (Standard Input / Output)
- **Primary Transport** for local tools and desktop IDE integrations.
- **CRITICAL NON-NEGOTIABLE**: The server must **ONLY write JSON-RPC messages to `stdout`**.
  - Any `print()`, `console.log()`, or debugging banner written to `stdout` corrupts the JSON-RPC message stream and crashes the client connection!
  - **All logging, debugging, and diagnostics MUST be written exclusively to `stderr`** (e.g. `sys.stderr.write()`, `console.error()`, or `eprintln!()`).

### 2. `SSE` (Server-Sent Events over HTTP)
- Used for remote tools or services hosted in separate containers.
- Client establishes SSE connection via `GET /sse` for downstream server messages.
- Client submits RPC calls via `POST /message?sessionId=<id>`.

---

## 5. Security & Isolation Guidelines

1. **Strict Input Sanitization**: Validate all file paths against path traversal attacks (`../`, symlinks). Restrict paths to the declared workspace root.
2. **Environment Variable Protection**: Never log or echo API tokens (`OPENAI_API_KEY`, `ADMIN_TOKEN`, etc.) in tool outputs or error messages.
3. **Context Caps**: Cap output strings at a maximum threshold (e.g. 50,000 characters or 100 rows) with truncation indicators to prevent context window saturation.

---

## 6. Progress Reporting & Cancellation Protocol

For operations taking $> 5\text{ seconds}$, servers should send out-of-band progress notifications:

### Progress Notification (`notifications/progress`)
```json
{
  "jsonrpc": "2.0",
  "method": "notifications/progress",
  "params": {
    "progressToken": "token-123",
    "progress": 45,
    "total": 100
  }
}
```

### Cancellation Listener (`notifications/cancelled`)
```json
{
  "jsonrpc": "2.0",
  "method": "notifications/cancelled",
  "params": {
    "requestId": 123,
    "reason": "User cancelled operation"
  }
}
```

---

## 7. Transparent Parameter Type Coercion

LLMs sometimes format numerical parameters as strings (e.g., `{"limit": "50"}`) or booleans as strings (`{"verbose": "true"}`). Servers should wrap argument dispatchers in defensive coercers:

```python
def coerce_param(param_value, expected_type):
    if expected_type == int and isinstance(param_value, str):
        try: return int(param_value)
        except ValueError: pass
    if expected_type == bool and isinstance(param_value, str):
        return param_value.lower() in ("true", "1", "yes")
    return param_value
```

---

## 8. Verification Tools

Audit and validate tool schemas and server configurations using:
- [`scripts/mcp_validator.py`](../scripts/mcp_validator.py)