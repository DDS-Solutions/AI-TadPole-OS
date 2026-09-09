---
name: gemma4-local
description: Provides core guidelines and prompt formatting patterns for interacting with local gemma4 models. Triggers when gemma4 is used as the local LLM backend.
when_to_use: "Trigger this skill when the active model ID contains 'gemma4' and is run locally (e.g., via Ollama or custom local server)."
version: 1.1.0
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / gemma4-local
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Token mismatch, raw thought leakage, or invalid tool string delimiters.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL:GEMMA4]`)

# Gemma 4 (gemma4) Local Model Interaction Protocol

> **Target Model Identifier**: Explicitly formatted as `gemma4` (no spaces/hyphens).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core token rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/prompt_templates_and_examples.md`](./references/prompt_templates_and_examples.md) | Multi-turn dialogue transcripts, raw tool JSON syntax, edge-case delimiters | Custom local server templates & raw token debugging |

---

## 1. Dialogue & Reasoning Control Tokens

| Control Token | Purpose | Positioning / Scope |
|---|---|---|
| `<|turn>system` ... `<turn|>` | System instruction container | Top of prompt |
| `<|think|>` | Activates the reasoning channel | Inside `<|turn>system` block |
| `<|channel>thought` ... `<channel|>` | Emits Chain-of-Thought reasoning | Precedes model response/tool call |
| `<|turn>user` / `<|turn>model` | Delimits conversation turns | Message boundaries |

---

## 2. Tool & Function Calling Tokens

| Token Pair | Function |
|---|---|
| `<|tool>` ... `<tool|>` | Declares available tools in system turn |
| `<|tool_call>` ... `<tool_call|>` | Model issues structured function call |
| `<|tool_response>` ... `<tool_response|>` | Ingests execution output back to model |

> [!IMPORTANT]
> **String Delimiter Policy**: Use `<|"|>` for **all string literals** inside tool blocks (e.g. `{path:<|"|>server-rs/src/main.rs<|"|>}`).

---

## 3. Standard Sampling Parameters

```json
{
  "temperature": 1.0,
  "top_p": 0.95,
  "top_k": 64
}
```