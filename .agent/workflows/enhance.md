> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[enhance]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Add or update features in existing application. Used for iterative development.
---

# /enhance - Update Application

$ARGUMENTS

---

## Task

This command adds features or makes updates to existing application.

### Steps:

1. **Sovereign Discovery**
   - Analyze Hub alignment (Gateway/Runner/Registry).
   - Check `parity_guard.py` for instructional drift.
   - Audit target technology (React 19 / Vite 8 / Tailwind v4 / Rust 1.8x).

2. **Architectural Hardening Plan**
   - Map dependencies in the 4-Pillars framework.
   - Identify shared state (Zustand) or schema implications (SQLite).

3. **High-Fidelity Implementation**
   - Apply changes using `Sovereign` code standards.
   - Verify with `python execution/verify_all.py .`.

4. **Telemetry Refresh**
   - Hot reload frontend (@ 5174).
   - Verify engine heartbeat (@ 8000).

---

## Usage Examples

```
/enhance add dark mode
/enhance build admin panel
/enhance integrate payment system
/enhance add search feature
/enhance edit profile page
/enhance make responsive
```

---

## Caution

- Get approval for major changes
- Warn on conflicting requests (e.g., "use Firebase" when project uses PostgreSQL)
- Commit each change with git

[//]: # (Metadata: [enhance])
