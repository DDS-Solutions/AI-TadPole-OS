> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: performance-profiling
description: Performance profiling principles. Measurement, analysis, and optimization techniques.
allowed-tools: Read, Glob, Grep, Bash
---

# Performance Profiling

**Measure, analyze, optimize.**

## Core Web Vitals
- **LCP** (Load): < 2.5s
- **INP** (Interactive): < 200ms
- **CLS** (Stability): < 0.1

## Profiling Workflow
1.  **BASELINE**: Measure first.
2.  **IDENTIFY**: Find bottleneck (Lighthouse, Bundle Analyzer, DevTools).
3.  **FIX**: Targeted change.
4.  **VALIDATE**: Confirm improvement.

## Analysis
- **Bundle**: Split large deps, dedupe, tree shake.
- **Runtime**: Long tasks (>50ms) block UI.
- **Memory**: Growing heap = leak.

## Quick Wins
1.  Compression (Gzip/Brotli).
2.  Lazy Load Images.
3.  Code Split Routes.
4.  Cache Static Assets.

---

## 🧠 Aletheia Reasoning Protocol (Optimization)

### 1. Generator (Hypothesis)
*   **Suspect**: "DB query slow?".
*   **Tool**: "`explain analyze`".
*   **Target**: "Reduce p99 by 50ms".

### 2. Verifier (Measurement)
*   **Baseline**: "Record metric BEFORE fix".
*   **Isolation**: "Network or CPU?".
*   **Significance**: "Is 1ms worth complexity?".

### 3. Reviser (Stabilization)
*   **Regression**: "Add test to prevent regression".
*   **Cache**: "Can't fix? Cache it.".

---

## 🛡️ Security & Safety Protocol (Profiling)

1.  **Timing Attacks**: No optimizations on auth checks.
2.  **Info Leak**: Don't publish profiles (PII/Env vars).
3.  **Prod Impact**: Don't crash prod with profiler overhead.
4.  **Sampling**: Use sampling on high traffic.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
