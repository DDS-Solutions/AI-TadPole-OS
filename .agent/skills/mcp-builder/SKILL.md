> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: mcp-builder
description: MCP server building principles. Tool design, resources.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# MCP Builder

**Connect AI to the world.**

## Core Concepts
- **Tools**: Executable functions (e.g., `get_weather`).
- **Resources**: Readable data (e.g., `file://logs`).
- **Prompts**: Templates.

## Tool Design
- **Single Purpose**: One thing well.
- **Schema**: Typed inputs, descriptions required.
- **Output**: Structured JSON, predictable.

## Resource Patterns
- **Static**: Config/Docs.
- **Dynamic**: Real-time generation.
- **URI**: `scheme://resource/param`.

---

## 🧠 Aletheia Reasoning Protocol (Tool Design)

### 1. Generator (Capability Definition)
*   **Atomic**: "Does this do ONE thing?".
*   **Naming**: "`create_record` > `make_stuff`".
*   **Argumentation**: "What does AI need to know?".

### 2. Verifier (Schema Validation)
*   **Types**: "String vs Number?".
*   **Ambiguity**: "Date format? Timezone?".
*   **Failure**: "Handle API downtime gracefully?".

### 3. Reviser (Simplification)
*   **Flatten**: "Avoid deep nesting.".
*   **Defaults**: "Allow omitting args where sensible.".

---

## 🛡️ Security & Safety Protocol (MCP)

1.  **Sandboxing**: Least privilege execution.
2.  **Validation**: No path traversal (`../../`).
3.  **Network**: SSRF protection (whitelist hosts).
4.  **Auth**: API keys stay in server env, NOT passed by LLM.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
