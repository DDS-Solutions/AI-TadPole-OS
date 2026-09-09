> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mcp-builder
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Malformed JSON-RPC payloads or transport disconnects.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[MCP_BUILDER]`)

# MCP Server Specifications & Transport Protocol (L3)

---

## 1. JSON-RPC 2.0 Request / Response Schema

```json
// Tool Call Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "read_file",
    "arguments": {
      "path": "server-rs/src/main.rs"
    }
  }
}

// Tool Call Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "fn main() { ... }"
      }
    ]
  }
}
```

---

## 2. Server Transport Selection

| Transport | Connection Model | Best For |
|---|---|---|
| **`stdio`** | Local subprocess piping via stdin/stdout | Desktop IDE plugins, CLI assistants |
| **`SSE` (Server-Sent Events)** | HTTP unidirectional streaming | Cloud tools, webhooks, lightweight servers |
| **`WebSocket`** | Bidirectional TCP frame streaming | Real-time swarms, bi-directional event buses |