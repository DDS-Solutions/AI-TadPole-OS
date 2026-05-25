> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.

---
name: ui-audit
description: Evaluating user interfaces for consistency, usability, and accessibility.
---

# UI Audit Protocol

A UI Audit ensures that the implemented product matches the design vision and industry standards.

## Architecture

```mermaid
graph TD
    S[Scope] --> V[Visual Check]
    V --> I[Interaction Check]
    I --> A[Accessibility Check]
    A --> R[Report]
```

### 1. Visual Consistency
- **Typography**: Are fonts/sizes consistent with the Design System?
- **Spacing**: Is padding/margin consistent (8pt grid)?
- **Color**: Are we using the defined palette?

### 2. Interaction Design
- **States**: Do buttons have hover, active, disabled states?
- **Feedback**: Does the user know their action succeeded or failed?
- **Responsiveness**: Does it break on mobile?

### 3. Accessibility (a11y)
- **Contrast**: Is text readable?
- **Keyboard Nav**: Can you use it without a mouse?
- **Screen Readers**: Do images have alt text?

## When to Use
- **Pre-Launch**: "Pixel Perfect" review.
- **Periodic Health Check**: Quarterly review of the entire app.

## Operational Principles
1. **Don't Trust Your Eyes**: Use tools (Inspect Element, Lighthouse).
2. **Mobile First**: If it works on mobile, it usually works on desktop.
3. **Errors are UX**: The text of an error message is part of the design.

[//]: # (Metadata: [ui_audit])

[//]: # (Metadata: [ui_audit])
