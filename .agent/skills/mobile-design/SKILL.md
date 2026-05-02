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
name: mobile-design
description: Mobile-first design thinking. Touch, performance, platform conventions.
allowed-tools: Read, Glob, Grep, Bash

command: ""
---

# Mobile Design System

**Touch-first. Battery-conscious. Platform-respectful.**

## 🔧 Runtime Scripts
`python scripts/mobile_audit.py <project_path>`

## ⛔ AI Mobile Sins (Anti-Patterns)
1.  **Perf**: `ScrollView` for lists (Use `FlatList`), Inline `renderItem`, `console.log` in prod.
2.  **UX**: Targets <44px, No loading/error states, Ignoring platform conventions.
3.  **Sec**: Token in `AsyncStorage` (Use `SecureStore`), Hardcoded keys.

## Platform Decision Matrix
- **Unify**: Business Logic, Data Layer.
- **Diverge**: Nav (Swipe vs Back Btn), Icons (SF vs Material), Typography.
- **Conventions**: iOS = Edge Swipe, Spinner. Android = Back Btn, Bar Progress.

## UX Psychology
- **Fitts' Law**: Closer + Bigger = Faster. Targets ≥44px.
- **Thumb Zone**: Primary actions at bottom. Destructive at top.
- **Load**: One task at a time. No hover states.

## Performance Rules
- **Lists**: `FlatList` + `React.memo` + `keyExtractor`.
- **Images**: Cached, resized.
- **Animation**: Native driver (transform/opacity only).

---

## 🧠 Aletheia Reasoning Protocol (Mobile)

### 1. Generator (Context Simulation)
*   **Env**: "Sun/Dark? Shaky train?".
*   **State**: "Low battery? Offline?".
*   **Input**: "Thumb? Voice?".

### 2. Verifier (Friction Audit)
*   **Reach**: "Hit 'Back' comfortably?".
*   **Speed**: "<2s load on 4G?".
*   **Feedback**: "Haptics present?".

### 3. Reviser (Native Polish)
*   **Conventions**: "iOS toggle on Android? No.".
*   **Nav**: "Hardware back button logic?".

---

## 🛡️ Security & Safety Protocol (Mobile)

1.  **Storage**: Keychain/Keystore ONLY for secrets.
2.  **Transport**: Cert Pinning recommended.
3.  **Privacy**: Blur screen in task switcher.
4.  **Perms**: Just-in-time requests.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
