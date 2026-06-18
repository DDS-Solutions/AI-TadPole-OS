> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills:Gemma4Local**
> - **Failure Path**: Token mismatch, raw thought leakage, or invalid tool string delimiters.
> - **Telemetry Link**: Search `[SKILL:GEMMA4]` in audit logs.
>
> ### AI Assist Note
> Core interaction rules for local Gemma 4 deployments.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: gemma4-local
description: Provides core guidelines and prompt formatting patterns for interacting with local gemma4 models. Triggers when gemma4 is used as the local LLM backend.
when_to_use: "Trigger this skill when the active model ID contains 'gemma4' and is run locally (e.g., via Ollama or custom local server)."
version: 1.0.0
---

# Gemma 4 (gemma4) Local Model Interaction Skill

> [!NOTE]
> Gemma 4 model identifiers are formatted without spaces or hyphens (specifically as **`gemma4`** in local environments and APIs).

This skill ensures that Tadpole OS agents interact correctly with local `gemma4` models, aligning with the model's specialized control tokens, reasoning channels, and tool-use lifecycle.

---

## 1. Dialogue Turn Formatting

gemma4 introduces turn-based dialogue delimiters. If interacting with the model via a raw prompt interface (or if configuring custom model templates), structure the conversation using:

- `<|turn>system` - Starts a system instruction block.
- `<|turn>user` - Starts a user message block.
- `<|turn>model` - Starts a model response block.
- `<turn|>` - Closes the current turn.

### Example Dialogue
```text
<|turn>system
You are a helpful assistant.<turn|>
<|turn>user
Hello.<turn|>
<|turn>model
Hello! How can I help you today?<turn|>
```

---

## 2. Thinking Mode (Reasoning Channel)

gemma4 features a built-in reasoning channel for Chain-of-Thought (CoT).

### Activation
To activate thinking mode, include the `<|think|>` control token inside the system instruction:
```text
<|turn>system
<|think|>You are a helpful assistant.<turn|>
```

> [!IMPORTANT]
> **Consolidation Rule**: The `<|think|>` token is designed to enable thinking mode at the conversation level. It must be consolidated into a single system turn alongside other system instructions (such as tool definitions) rather than being split across multiple turns.

### Thinking Channel Tokens
When thinking mode is active, the model generates thoughts in the `<|channel>thought` block before outputting its final response or tool call:
```text
<|turn>model
<|channel>thought
[Thinking tokens generated here...]
<channel|>This is the final response provided to the user.<turn|>
```

---

## 3. Tool Use & Function Calling

gemma4 utilizes six special control tokens to manage tool usage:

| Token Pair | Purpose |
|---|---|
| `<|tool>` `<tool|>` | Defines a tool schema within the system prompt. |
| `<|tool_call>` `<tool_call|>` | Model requests a tool execution. |
| `<|tool_response>` `<tool_response|>` | Feeds the tool execution result back to the model. |

> [!NOTE]
> `<|tool_response>` acts as an additional stop sequence for the inference engine.

### String Delimiter Policy: `<|"|>`
> [!IMPORTANT]
> **CRITICAL RULE**: A single token, `<|"|>`, must be used as the string delimiter for **all string values** inside structured tool data blocks (declarations, calls, and responses). This prevents special characters from breaking parsing syntax.

- **Incorrect**: `{location: "London"}`
- **Correct**: `{location:<|"|>London<|"|>}`

### Example Tool Loop Interaction

1. **System Prompt (Tool Declaration)**:
   ```text
   <|turn>system
   <|think|>You are a helpful assistant.<|tool>declaration:get_current_weather{location:{type:<|"|>string<|"|>}}<tool|><turn|>
   ```
2. **User Inquiry**:
   ```text
   <|turn>user
   What's the weather in London?<turn|>
   ```
3. **Model Thought and Tool Call**:
   ```text
   <|turn>model
   <|channel>thought
   User wants weather info. I need to use get_current_weather tool.
   <channel|><|tool_call>call:get_current_weather{location:<|"|>London<|"|>}<tool_call|><|tool_response>
   ```
4. **Tool Execution & Injecting Response**:
   ```text
   <|turn>model
   <|tool_call>call:get_current_weather{location:<|"|>London<|"|>}<tool_call|><|tool_response>response:get_current_weather{temperature:15,weather:<|"|>sunny<|"|>}<tool_response|>
   ```
5. **Final Model Response**:
   ```text
   The temperature in London is 15 degrees and it is sunny.<turn|>
   ```

### JSON Chat History Structure
Your application should parse the model's response to extract the function name and arguments, execute the function, and then append the `tool_calls` and `tool_responses` to the chat history under the `assistant` role:
```json
[
  {
    "role": "system",
    "content": "You are a helpful assistant."
  },
  {
    "role": "user",
    "content": "What's the weather in London?"
  },
  {
    "role": "assistant",
    "tool_calls": [
      {
        "function": {
          "name": "get_current_weather",
          "arguments": {
            "location": "London"
          }
        }
      }
    ],
    "tool_responses": [
      {
        "name": "get_current_weather",
        "response": {
          "temperature": 15,
          "weather": "sunny"
        }
      }
    ],
    "content": "The temperature in London is 15 degrees and it is sunny."
  }
]
```

> [!NOTE]
> **API Compatibility Note**: The unified `assistant` JSON structure above represents how the Google GenAI SDK internally structures a tool loop. In standard OpenAI-compatible server APIs (like Ollama or vLLM compatible interfaces), tool execution is represented using separate messages with the `tool` role:
> ```json
> [
>   {
>     "role": "user",
>     "content": "What's the weather in London?"
>   },
>   {
>     "role": "assistant",
>     "tool_calls": [
>       {
>         "id": "call_123",
>         "type": "function",
>         "function": { "name": "get_current_weather", "arguments": "{\"location\": \"London\"}" }
>       }
>     ]
>   },
>   {
>     "role": "tool",
>     "tool_call_id": "call_123",
>     "name": "get_current_weather",
>     "content": "{\"temperature\": 15, \"weather\": \"sunny\"}"
>   },
>   {
>     "role": "assistant",
>     "content": "The temperature in London is 15 degrees and it is sunny."
>   }
> ]
> ```

---

## 4. Multimodal Inputs (Image & Audio)

Gemma 4 is natively multimodal and defines specific control tokens for handling image and audio embeddings.

### Multimodal Tokens

| Token Pair | Purpose |
|---|---|
| `<|image>` `<image|>` | Delimits image soft embeddings after tokenization. |
| `<|audio>` `<audio|>` | Delimits audio soft embeddings after tokenization. |
| `<|image|>` | Special placeholder token indicating where image embeddings will be inserted. |
| `<|audio|>` | Special placeholder token indicating where audio embeddings will be inserted. |

### Prompt Structure Example
When formulating multimodal prompts, place placeholder tokens at the exact position in the conversation text where the media belongs:
```text
<|turn>user
Describe this image: <|image|> And transcribe this audio segment: <|audio|><turn|>
<|turn>model
```

### Audio Task Prompt Templates
To optimize local speech tasks, prepend specific instructions to the model:

* **Audio Speech Recognition (ASR)**:
  ```text
  Transcribe the following speech segment in {LANGUAGE} into {LANGUAGE} text. Follow these specific instructions for formatting the answer:
  - Only output the transcription, with no newlines.
  - When transcribing numbers, write the digits (e.g., write 1.7 instead of "one point seven", and write 3 instead of "three").
  ```
* **Automatic Speech Translation (AST)**:
  ```text
  Transcribe the following speech segment in {SOURCE_LANGUAGE}, then translate it into {TARGET_LANGUAGE}. When formatting the answer, first output the transcription in {SOURCE_LANGUAGE}, then one newline, then output the string '{TARGET_LANGUAGE}: ', followed by the translation in {TARGET_LANGUAGE}.
  ```

---

## 5. Context & History Management

Managing generated thoughts is crucial for context window performance.

### Standard Multi-Turn Conversations
- **Rule**: You must **strip** the previous turns' thoughts (the text between `<|channel>thought` and `<channel|>`) before sending the conversation history back to the model. Leaving previous raw thoughts causes token bloat and degrades performance.
- To disable thinking mode mid-conversation, remove the `<|think|>` token from the system block when stripping thoughts.

### Tool Call Loop Exception
- **Rule**: During a single assistant turn involving active tool calls, thoughts must **NOT** be removed between the tool calls. Keep them intact until the final turn response is completed.

### Preventing Loop Traps in Long-Running Agents
- To prevent the model from entering cyclical loops while keeping reasoning context:
  1. Extract and summarize the model's previous thoughts.
  2. Inject the summarized thoughts as standard text into the system prompt.
  3. Since gemma4 has no fixed schema for injected summaries, use a clear header like `### Summarized Prior Reasoning:` to instruct the model.

---

## 6. Adaptive Thought Efficiency

While "thinking" in Gemma 4 is officially supported as an ON or OFF boolean feature, the model has exceptionally strong instruction-following capabilities that allow you to modulate its thinking behavior dynamically.

Rather than relying on a hardcoded framework parameter for "high" or "low" thinking, you can use System Instructions (SI) to guide the model into a reduced thinking mode. By explicitly instructing the model to think efficiently or at a lower depth (a "LOW" thinking instruction), you can achieve adaptive thought efficiency.

- **Reduced Cost**: Applying a "LOW" thinking System Instruction can reduce the number of thinking tokens generated by approximately 20%.
- **Proof of Concept**: Because this behavior is a byproduct of instructability rather than custom training, there is no single "perfect" prompt. Custom SIs should be tailored to balance latency, cost, and output quality.
- **Example SI Addition**:
  ```text
  "CRITICAL: Keep your thinking process efficient and concise. Focus only on the core logic transitions."
  ```

---

## 7. Integration & Stabilization Notes

- **Reasoning Channel Visibility**: The `<|channel>thought` and `<channel|>` tokens are used for Chain-of-Thought (CoT). In user-facing environments, this content should typically be stripped or hidden from the final user interface.
- **Large Model Stabilization (e.g., `gemma-4-26B-A4B-it`, `gemma-4-31B-it`)**: Larger models may occasionally generate a thought channel even when thinking mode is explicitly turned off. To stabilize behavior in these edge cases, consider adding an empty thinking token to the prompt.
- **Fine-Tuning with No-Thinking Datasets**: When fine-tuning `gemma-4-26B-A4B-it` or `gemma-4-31B-it` with a dataset that does not include thinking, you can achieve better results by adding the empty channel to your training prompts:
  ```text
  <|turn>model
  <|channel>thought
  <channel|>
  ```

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
