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
name: mobile-games
description: Mobile game development principles. Touch input, battery, performance, app stores.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# Mobile Game Development

> Platform constraints and optimization principles.

---

## 1. Platform Considerations

### Key Constraints

| Constraint | Strategy |
|------------|----------|
| **Touch input** | Large hit areas, gestures |
| **Battery** | Limit CPU/GPU usage |
| **Thermal** | Throttle when hot |
| **Screen size** | Responsive UI |
| **Interruptions** | Pause on background |

---

## 2. Touch Input Principles

### Touch vs Controller

| Touch | Desktop/Console |
|-------|-----------------|
| Imprecise | Precise |
| Occludes screen | No occlusion |
| Limited buttons | Many buttons |
| Gestures available | Buttons/sticks |

### Best Practices

- Minimum touch target: 44x44 points
- Visual feedback on touch
- Avoid precise timing requirements
- Support both portrait and landscape

---

## 3. Performance Targets

### Thermal Management

| Action | Trigger |
|--------|---------|
| Reduce quality | Device warm |
| Limit FPS | Device hot |
| Pause effects | Critical temp |

### Battery Optimization

- 30 FPS often sufficient
- Sleep when paused
- Minimize GPS/network
- Dark mode saves OLED battery

---

## 4. App Store Requirements

### iOS (App Store)

| Requirement | Note |
|-------------|------|
| Privacy labels | Required |
| Account deletion | If account creation exists |
| Screenshots | For all device sizes |

### Android (Google Play)

| Requirement | Note |
|-------------|------|
| Target API | Current year's SDK |
| 64-bit | Required |
| App bundles | Recommended |

---

## 5. Monetization Models

| Model | Best For |
|-------|----------|
| **Premium** | Quality games, loyal audience |
| **Free + IAP** | Casual, progression-based |
| **Ads** | Hyper-casual, high volume |
| **Subscription** | Content updates, multiplayer |

---

## 6. Anti-Patterns

| ❌ Don't | ✅ Do |
|----------|-------|
| Desktop controls on mobile | Design for touch |
| Ignore battery drain | Monitor thermals |
| Force landscape | Support player preference |
| Always-on network | Cache and sync |

---

> **Remember:** Mobile is the most constrained platform. Respect battery and attention.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
