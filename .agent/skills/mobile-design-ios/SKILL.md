---
name: mobile-design-ios
description: Mobile-first iOS design thinking, SwiftUI, UIKit, Apple Human Interface Guidelines (HIG), SF Symbols, NavigationStack/SplitView, @Observable macro, SwiftData/Core Data, and iOS platform conventions.
when_to_use: "When designing or building iOS mobile app interfaces, SwiftUI UIs, or native Swift iOS components. NOT for Android or desktop web apps."
allowed-tools: Read, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# iOS Mobile Design System (SwiftUI & Apple HIG)

> **Philosophy:** Touch-first. Fluid 120Hz ProMotion. Apple HIG Native. Battery & Memory conscious.
> **Core Principle:** iOS interfaces must strictly adhere to Apple Human Interface Guidelines, fluid gestures, SF Symbols, and modern SwiftUI architecture patterns.

---

## 🔴 MANDATORY: Read Reference Files Before Working!

**⛔ DO NOT start iOS development until you read the relevant reference files:**

| File | Content | Status |
|------|---------|--------|
| **[platform-ios.md](platform-ios.md)** | **SwiftUI Architecture, `@Observable`, View Lifecycles** | **⬜ CRITICAL FIRST** |
| **[ios-hig-guidelines.md](ios-hig-guidelines.md)** | **Apple HIG, 44pt touch targets, Dynamic Type, SF Symbols** | **⬜ CRITICAL** |
| **[ios-navigation.md](ios-navigation.md)** | **`NavigationStack`, `NavigationSplitView`, Sheets, Gestures** | **⬜ CRITICAL** |
| **[ios-performance-testing.md](ios-performance-testing.md)** | **Instruments, 120Hz render loop, Swift Testing (@Test)** | **⬜ CRITICAL** |

---

## ⚡ Key iOS Design & Architecture Standards

### 1. Apple Human Interface Guidelines (HIG)
* **Touch Targets:** Minimum 44x44pt hit area for all interactive controls (`Button`, `Toggle`, `TextField`).
* **Typography:** Always use Dynamic Type styles (`.title`, `.body`, `.headline`, `.caption`) so text scales automatically with system font settings.
* **Icons:** Use **SF Symbols 5+** (`Image(systemName: "shield.fill")`) for standard icons.
* **Materials & Vibrancy:** Use system background materials (`.ultraThinMaterial`, `.regularMaterial`) and system colors (`Color(uiColor: .systemBackground)` or `Color.accentColor`) for dark/light mode compatibility.

### 2. Modern SwiftUI Architecture (iOS 17+)
* **Observation Framework:** Use `@Observable` class macro instead of legacy `ObservableObject` / `@Published` to minimize re-renders and enable fine-grained view tracking.
* **Navigation:** Use `NavigationStack` with `navigationDestination(for:)` for type-safe stack navigation. Use `NavigationSplitView` for iPad / multi-column layouts.
* **Presentation:** Use `.sheet` for modal tasks, `.fullScreenCover` for imperative modal flows, and `.confirmationDialog` for destructive actions.
* **Feedback:** Add haptic feedback using `SensoryFeedback` modifier (e.g. `.sensoryFeedback(.impact, trigger: isApproved)`).

---

## ⛔ AI iOS ANTI-PATTERNS (YASAK LİSTESİ)

| ❌ NEVER DO | Why It's Wrong | ✅ ALWAYS DO |
|-------------|----------------|--------------|
| **Hardcoded frame sizes** | Breaks on different iPhone/iPad models | Use relative spacing, `Layout`, or `.frame(maxWidth: .infinity)` |
| **Custom Back Buttons without edge swipe** | Breaks standard iOS interactive pop gesture | Retain native `NavigationStack` back button & gesture |
| **`ObservableObject` on iOS 17+** | Re-evaluates whole object on any published property change | Use `@Observable` macro for fine-grained dependency tracking |
| **Ignoring Dark Mode** | App looks unreadable in Light/Dark transitions | Use semantic system colors (`Color.primary`, `Color(uiColor: .secondarySystemBackground)`) |
| **Storing Secrets in `UserDefaults`** | Plain text exposure | Store auth tokens in **iOS Keychain** (`SecItemAdd`) |

---

## 🛠️ Verification & Build Checks

When verifying iOS code:
```bash
# Build & Test with Xcode Command Line Tools
xcodebuild -scheme AppScheme -destination 'platform=iOS Simulator,name=iPhone 15' build test
```
