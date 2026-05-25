> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Mythos Recurrent Reasoning Integration

## Overview
This document outlines the implementation of **Mythos-inspired Recurrent Reasoning** within the Tadpole OS agent engine. This architecture allows agents to perform multiple "internal turns" (recurrent inference steps) before committing to an external action or final response.

## Core Components

### 1. Recurrent Intelligence Loop
Located in `server-rs/src/agent/runner/intelligence.rs`, the `execute_intelligence_loop` function utilizes a `while` loop that continues until:
- The maximum `reasoning_depth` (1-16) is reached.
- A **Halting Signal** is detected.
- An external tool call or mission completion is triggered.
- **Financial Fail-safe**: The mission budget is exceeded mid-recurrence.

### 2. Hybrid Halting Mechanism
To support **Adaptive Computation Time (ACT)**, agents can signal early completion via:
- **XML Signaling**: Adding `<halting_signal/>` or `<halt/>` to their internal monologue.
- **Tool Signaling**: Calling `set_confidence(score)`. If `score >= act_threshold` (default 0.95), the loop terminates.

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
| `act_threshold` | `f32` | 0.0-1.0 | Confidence level for ACT halting. |

## Implementation Details
- **Backend**: `EngineAgent` and `ModelConfig` (`types.rs`).
- **Logic**: `execute_intelligence_loop` (`intelligence.rs`).
- **Parity**: Verified via `test_agent_serialization_parity`.

[//]: # (Metadata: [Mythos_Integration])

[//]: # (Metadata: [Mythos_Integration])
