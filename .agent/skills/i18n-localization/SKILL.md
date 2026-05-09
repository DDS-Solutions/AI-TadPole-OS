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
name: i18n-localization
description: Internationalization patterns. Detecting hardcoded strings, managing translations, locale files.
allowed-tools: Read, Glob, Grep

command: ""
---

# i18n & Localization

**Language is culture, not just words.**

## Core Concepts
- **i18n** (Enable) vs **L10n** (Translate).
- **Structure**: `locales/{lang}/{namespace}.json`.
- **RTL**: Plan for right-to-left layout.

---

## 🧠 Aletheia Reasoning Protocol (Localization)

### 1. Generator (Visual Expansion)
*   **Length**: "German is 30% longer. Will the button break?".
*   **Direction**: "Arabic is RTL. Will the sidebar flip?".
*   **Context**: "Is 'Date' a fruit or a meeting?".

### 2. Verifier (Cultural Audit)
*   **Colors**: "Red means lucky in China, danger in US.".
*   **Formats**: "MM/DD/YYYY vs DD/MM/YYYY".
*   **Hardcoding**: "Grep for raw strings".

### 3. Reviser (Key Management)
*   **Hierarchy**: "Nest keys by page/feature".
*   **Plurals**: "Use ICU format".

---

## 🛡️ Security & Safety Protocol (i18n)

1.  **XSS**: Sanitize translations (No `<script>`).
2.  **Interpolation**: Prevent injection via variables.
3.  **Locale Spoofing**: Validate locale param against allowlist.
4.  **Info Leak**: Consistent error messages across languages.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
