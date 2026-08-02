---
name: digital-twin-voice
description: Extracts SMB founder/brand tone, formality, and communication phrasing to calibrate AI Digital Twin persona artifacts.
when_to_use: "Use when calibrating the AI Digital Twin's tone, customer service style, email voice, or brand persona."
allowed-tools: Read, Glob, Grep, Write
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for extracting SMB brand voice and calibrating Digital Twin persona system prompts.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

# Digital Twin Persona & Voice Calibration

> Purpose: Capture the authentic brand voice of an SMB owner so customer interactions feel natural and on-brand.

---

## Calibration Protocol

1. **Sample Ingestion**:
   Analyze sample communications (emails, customer chat logs, FAQs, founder notes).
2. **Trait Extraction**:
   Identify core tone metrics:
   - **Formality**: (Casual $\leftrightarrow$ Formal)
   - **Enthusiasm**: (Reserved $\leftrightarrow$ Energetic)
   - **Directness**: (Empathetic/Verbose $\leftrightarrow$ Concise/Direct)
   - **Brand Jargon**: Signature phrases, greetings, and sign-offs.
3. **Artifact Generation**:
   Generate the calibrated persona system prompt and save to `.tmp/twin-persona.md`.
