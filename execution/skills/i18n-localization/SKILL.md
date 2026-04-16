---
name: i18n-localization
description: Internationalization patterns. Detecting hardcoded strings, managing translations, locale files.
allowed-tools: Read, Glob, Grep
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
