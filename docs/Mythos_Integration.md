
# Mythos Recurrent Reasoning Integration

## Overview
This document outlines the implementation of **Mythos-inspired Recurrent Reasoning** within the Tadpole OS agent engine. This architecture allows agents to perform multiple "internal turns" (recurrent inference steps) before committing to an external action or final response.

## Core Components

### 1. Recurrent Intelligence Loop
Located under `server-rs/src/agent/runner/intelligence/` (module directory; not a single `intelligence.rs` file), the recurrent loop utilizes a `while` that continues until:
- The maximum `reasoning_depth` (1-16) is reached.
- A **Halting Signal** is detected.
- An external tool call or mission completion is triggered.
- **Financial Fail-safe**: The mission budget is exceeded mid-recurrence.

### 2. Hybrid Halting Mechanism
To support **Adaptive Computation Time (ACT)**, agents can signal early completion via:
- **XML Signaling**: Adding `<halting_signal/>` or `<halt/>` to their internal monologue.
- **Tool Signaling**: Calling `set_confidence(score)`. If `score >= act_threshold`, the loop terminates.
  - **Dual defaults (tip `bbcf0d4`)**: runner/`context.rs` and `runner/mod.rs` default **`0.9`** when unset; `types/agent.rs` `unwrap_or` uses **`0.95`**. Prefer setting `act_threshold` explicitly on the model slot to avoid ambiguity.

### 3. Neural Pulse Telemetry
- **State Synchronization**: `current_reasoning_turn` is updated in the global `AgentRegistry` at 10Hz.
- **RAII Safety**: A `ReasoningTurnGuard` ensures the registry turn indicator is reset to `0` upon loop exit (success or failure).

## Scaling & Hygiene

### 1. Monologue Compression
To prevent context window overflow during deep reasoning (depth > 8), the engine implements **Recursive Summarization**. If the internal monologue exceeds 8,192 characters, it is summarized into a technical "Consolidated Reasoning" block.

### 2. Output Scrubbing
Internal control markers (`<halting_signal/>`, `<thinking>`) are automatically pruned via `scrub_mythos_tags` before being promoted to the active conversation or user dashboard.

## Configuration
| Parameter | Type | Range | Description |
|:--- |:--- |:--- |:--- |
| `reasoning_depth` | `u32` | 1-16 | Maximum internal turns. |
| `act_threshold` | `f32` | 0.0-1.0 | Confidence level for ACT halting. Dual defaults: runner **0.9** vs agent-types **0.95** when unset. |

## Implementation Details
- **Backend**: `EngineAgent` and `ModelConfig` (`types.rs`).
- **Logic**: recurrent intelligence loop under `intelligence/` (`turn.rs`, `mod.rs`, …).
- **Parity**: Verified via `test_agent_serialization_parity`.

[//]: # (Metadata: [Mythos_Integration])
