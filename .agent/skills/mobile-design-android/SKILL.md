---
name: mobile-design-android
description: "Mobile-first Android design thinking, Jetpack Compose, Material 3 (M3), Compose Navigation 3, Compose Material 3 Adaptive layouts, StateFlow / ViewModel state management, and Android platform conventions. Synchronized with official Google Developer documentation."
when_to_use: "When designing or building Android mobile app interfaces, Jetpack Compose UIs, or native Kotlin Android components. NOT for iOS or desktop web apps."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mobile-design-android
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy XML patterns, or mobile ergonomic regressions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Android Mobile Design System (Jetpack Compose & M3)

> **Philosophy**: Touch-first, battery-conscious, Material 3 native, offline-capable.  
> **Core Principle**: Android is NOT a small desktop or an iOS clone. Adhere strictly to Google Material 3 guidelines and Jetpack Compose patterns.  
> **Workflow Bindings**: Integrated into [`/create`](../../workflows/create.md) and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/platform-android.md`](./references/platform-android.md) | Material Design 3 tokens, Jetpack Compose widgets, M3 Adaptive layouts | Android UI & Compose architecture |
| [`references/touch-psychology.md`](./references/touch-psychology.md) | Thumb zones, 48dp touch targets, gesture navigation, haptics | Touch accessibility & layout ergonomics |
| [`references/mobile-performance.md`](./references/mobile-performance.md) | 60fps frame budgeting, LazyColumn optimization, memory leaks | Performance profiling & scroll jank |
| [`references/mobile-navigation.md`](./references/mobile-navigation.md) | Compose Navigation 3, deep links, backstack handling | Route flows & screen transitions |
| [`references/mobile-color-system.md`](./references/mobile-color-system.md) | Dynamic Color (Material You), AMOLED dark theme, high contrast | Palette generation & themes |
| [`references/mobile-backend.md`](./references/mobile-backend.md) | Offline caching (Room/DataStore), push notifications, background sync | Data persistence & sync engine |
| [`references/mobile-testing.md`](./references/mobile-testing.md) | Compose UI test rules, unit testing ViewModels, mock engines | Testing & verification |
| [`scripts/mobile_audit.py`](./scripts/mobile_audit.py) | Automated touch target, layout spacing, and accessibility auditor | Auditing mobile view implementations |

---

## ⚠️ 1. Mandatory Socratic Gate (Ask Before Assuming)

If Android specifications are unspecified, **STOP and ask**:
1. **Layout Scope**: *"Is this phone-only or should we support adaptive layouts (tablets/foldables)?"*
2. **Offline Requirements**: *"Does this screen require offline-first caching (Room/DataStore) or live network only?"*
3. **Dynamic Color**: *"Should the UI use user-generated Dynamic Color (Material You) or strict brand palette tokens?"*

---

## 🚫 2. Critical Mobile Anti-Patterns (Never Do)

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Touch Targets < 48dp** | Buttons smaller than 48dp cause missed taps and rage clicks. | Enforce minimum 48dp $\times$ 48dp touch targets with $\ge 8$dp spacing. |
| **Blocking Main Thread** | Running Room queries or network requests on UI thread causes ANRs. | Use `Dispatchers.IO` and Kotlin coroutines / Flows. |
| **Unremembered Lambdas** | Instantiating lambdas or objects in `@Composable` triggers unnecessary recomposition. | Wrap objects and lambda handlers in `remember`. |
| **Nested Scroll Containers** | Vertically nested scroll containers inside `LazyColumn` cause scroll freezes. | Use fixed heights or single unified lazy lists. |
| **Hardcoded Text** | Hardcoding UI strings prevents localization. | Use `stringResource(R.string.key)` or localized resource maps. |

---

## 🛠️ 3. Execution & Verification Workflow

```
[1. DESIGN] ───> [2. COMPOSE] ───> [3. ADAPTIVE] ───> [4. AUDIT]
 M3 color tokens  Jetpack Compose   WindowSizeClass     mobile_audit.py
 & typography     clean architecture adaptive layout    & Compose tests
```

1. **Design**: Align with M3 color schemes and typography scales.
2. **Compose**: Build declarative Jetpack Compose UI components with clear StateFlow bindings.
3. **Adaptive**: Support varying form factors using `WindowSizeClass` (Compact, Medium, Expanded).
4. **Audit**: Run `python .agent/skills/mobile-design-android/scripts/mobile_audit.py` to verify touch bounds.

---

## 🛠️ 4. Deterministic Verification Gate

Validate mobile UI constraints and accessibility:

```powershell
# 1. Run built-in self-tests
python .agent/skills/mobile-design-android/scripts/mobile_audit.py --test

# 2. Audit a target directory for mobile touch targets and ergonomics
python .agent/skills/mobile-design-android/scripts/mobile_audit.py path/to/mobile/components
```