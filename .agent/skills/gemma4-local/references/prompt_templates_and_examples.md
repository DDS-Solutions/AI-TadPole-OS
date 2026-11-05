> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / gemma4-local
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Delimiter syntax mismatch or raw token leakage.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL:GEMMA4]`)

# Gemma 4 Prompt Delimiters & Tool Loop Examples (L3)

---

## 1. Complete Turn-Based Tool Interaction Template

```text
<|turn>system
<|think|>You are an expert developer assistant.<|tool>declaration:read_file{path:{type:<|"|>string<|"|>}}<tool|><turn|>
<|turn>user
Read the file server-rs/src/main.rs<turn|>
<|turn>model
<|channel>thought
I need to inspect main.rs to understand the system initialization sequence.
<channel|><|tool_call>call:read_file{path:<|"|>server-rs/src/main.rs<|"|>}<tool_call|><turn|>
<|turn>user
<|tool_response>{"content": "fn main() { println!(\"Starting Tadpole OS\"); }"}<turn|>
<|turn>model
<|channel>thought
The file content has been received. I will now explain the entrypoint to the user.
<channel|>The `main.rs` file contains the primary server entrypoint.<turn|>
```

---

## 2. String Delimiter Enforcement (`<|"|>`)

When generating or parsing tool declarations, calls, and responses:
- **Always delimit strings with `<|"|>`**: `{path:<|"|>data/tadpole.db<|"|>}`
- **Do NOT use standard double quotes** inside tool call blocks.