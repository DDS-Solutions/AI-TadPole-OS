---
name: mobile-design-android
description: Mobile-first Android design thinking, Jetpack Compose, Material 3 (M3), Compose Navigation 3, Compose Material 3 Adaptive layouts, StateFlow / ViewModel state management, and Android platform conventions. Synchronized with official Google Developer documentation.
when_to_use: "When designing or building Android mobile app interfaces, Jetpack Compose UIs, or native Kotlin Android components. NOT for iOS or desktop web apps."
allowed-tools: Read, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mobile-design-android
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Android Mobile Design System (Jetpack Compose & M3)

> **Philosophy:** Touch-first. Battery-conscious. Material 3 native. Offline-capable.
> **Core Principle:** Android is NOT a small desktop or an iOS clone. Adhere strictly to Google Material 3 guidelines and Jetpack Compose patterns.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core logic below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`platform-android.md`](./platform-android.md) | Material Design 3 tokens, Jetpack Compose widgets, M3 Adaptive layouts | Android UI & Compose architecture |
| [`touch-psychology.md`](./touch-psychology.md) | Thumb zones, 48dp touch targets, gesture navigation, haptics | Touch accessibility & layout ergonomic |
| [`mobile-performance.md`](./mobile-performance.md) | 60fps frame budgeting, LazyColumn optimization, memory leaks | Performance profiling & scroll jank |
| [`mobile-navigation.md`](./mobile-navigation.md) | Compose Navigation 3, deep links, backstack handling | Route flows & screen transitions |
| [`mobile-color-system.md`](./mobile-color-system.md) | Dynamic Color (Material You), AMOLED dark theme, high contrast | Palette generation & themes |
| [`mobile-backend.md`](./mobile-backend.md) | Offline caching (Room/DataStore), push notifications, background sync | Data persistence & sync engine |
| [`mobile-testing.md`](./mobile-testing.md) | Compose UI test rules, unit testing ViewModels, mock engines | Testing & verification |

---

## ⚠️ 1. Mandatory Socratic Gate (Ask Before Assuming)

If Android specifications are unspecified, **STOP and ask**:
1. **Layout Scope**: *"Is this phone-only or should we support adaptive layouts (tablets/foldables)?"*
2. **Offline Requirements**: *"Does this screen require offline-first caching (Room/DataStore) or live network only?"*
3. **Dynamic Color**: *"Should the UI use user-generated Dynamic Color (Material You) or strict brand palette tokens?"*

---

## 🚫 2. Critical Mobile Anti-Patterns (Never Do)

1. **Touch Targets < 48dp**: Never create interactive buttons smaller than 48dp $\times$ 48dp (minimum 8dp spacing).
2. **Blocking Main / UI Thread**: Never execute Room queries or network calls on `Dispatchers.Main`.
3. **Heavy Composition Re-evaluations**: Avoid instantiating lambda listeners or objects inside `@Composable` functions without `remember`.
4. **Nested Scrollable Containers**: Never nest vertical scrolling containers inside `LazyColumn` without fixed heights.
5. **Hardcoded Strings**: Never hardcode user-facing strings; use `stringResource(R.string.key)`.

---

## 🛠️ 3. Execution & Verification Workflow

```
1. DESIGN   ➔ Align with M3 color schemes and typography scales.
2. COMPOSE  ➔ Build declarative composables with ViewModel StateFlows.
3. AUDIT    ➔ Run `python scripts/mobile_audit.py <project_path>`.
4. VERIFY   ➔ Confirm 48dp touch targets, dark theme contrast, and smooth 60fps scroll.
```