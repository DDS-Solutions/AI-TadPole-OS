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

### 1. Generator (Skill Definition)
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
