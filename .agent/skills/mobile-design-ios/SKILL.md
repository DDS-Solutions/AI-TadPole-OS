---
name: mobile-design-ios
description: "Mobile-first iOS design thinking, SwiftUI, UIKit, Apple Human Interface Guidelines (HIG), SF Symbols, NavigationStack/SplitView, @Observable macro, SwiftData/Core Data, and iOS platform conventions."
when_to_use: "When designing or building iOS mobile app interfaces, SwiftUI UIs, or native Swift iOS components. NOT for Android or desktop web apps."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mobile-design-ios
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy UIKit/ObservableObject patterns, or Apple HIG non-compliance.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# iOS Mobile Design System (SwiftUI & Apple HIG)

> **Philosophy**: Touch-first, fluid 120Hz ProMotion, Apple HIG Native, battery & memory conscious.  
> **Core Principle**: iOS interfaces must strictly adhere to Apple Human Interface Guidelines, fluid gestures, SF Symbols, and modern SwiftUI architecture patterns.  
> **Workflow Bindings**: Integrated into [`/create`](../../workflows/create.md) and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign iOS rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/platform-ios.md`](./references/platform-ios.md) | SwiftUI Architecture, `@Observable` macro, View Lifecycles | SwiftUI core state & view implementation |
| [`references/ios-hig-guidelines.md`](./references/ios-hig-guidelines.md) | Apple HIG, 44pt touch targets, Dynamic Type, SF Symbols 5+ | Layout ergonomics, fonts, icons, styling |
| [`references/ios-navigation.md`](./references/ios-navigation.md) | `NavigationStack`, `NavigationSplitView`, Sheets, Gestures | Stack routing, iPad layouts, modal flows |
| [`references/ios-performance-testing.md`](./references/ios-performance-testing.md) | Instruments, 120Hz render loop, Swift Testing (`@Test`) | Performance tuning & unit test suites |
| [`scripts/ios_validator.py`](./scripts/ios_validator.py) | Automated static auditor for HIG targets, Keychain security, and modern `@Observable` | Static verification of Swift code |

---

## ⚠️ 1. Mandatory Socratic Gate (Ask Before Assuming)

Before implementing iOS views or architectural flows, verify:
1. **Target Form Factor**: *"Is this designed for iPhone only or multi-column iPad (`NavigationSplitView`)?"*
2. **Persistence Layer**: *"Should state be stored in SwiftData, Core Data, or purely ephemeral memory?"*
3. **Minimum iOS Target**: *"Is the deployment target iOS 17+ (allowing `@Observable` macro) or legacy iOS 16?"*

---

## 🚫 2. Critical iOS Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Touch Targets < 44pt** | Elements smaller than 44x44pt fail App Store review and frustrate users. | Enforce minimum 44pt hit target via `.frame(minWidth: 44, minHeight: 44)`. |
| **`ObservableObject` on iOS 17+** | Triggers massive view re-evaluations across entire object graphs. | Use the `@Observable` macro for fine-grained dependency tracking. |
| **Custom Back Buttons** | Overriding back button without restoring interactive swipe gestures. | Keep native `NavigationStack` back button or attach custom drag gesture. |
| **Secrets in `UserDefaults`** | Plaintext token storage exposes sensitive credentials to backups and dumps. | Store API keys and session tokens in **iOS Keychain** (`SecItemAdd`). |
| **Fixed Hardcoded Dimensions** | Static pixel/point dimensions clip on smaller screens or Dynamic Type. | Use dynamic layouts, `.frame(maxWidth: .infinity)`, and Dynamic Type sizes. |

---

## 🛠️ 3. Execution & Verification Workflow

```
[1. SPECIFY] ───> [2. SWIFTUI] ───> [3. HIG CHECK] ───> [4. VALIDATE]
 HIG Tokens &      @Observable &     Dynamic Type,       ios_validator.py
 SF Symbols        NavigationStack   44pt touch area     & Swift tests
```

1. **Specify**: Select SF Symbols 5+ and Apple semantic colors (`Color(uiColor: .systemBackground)`).
2. **SwiftUI**: Implement views using `@Observable` macro and type-safe `NavigationStack`.
3. **HIG Check**: Ensure hit testing areas are $\ge 44$pt and text respects Dynamic Type scaling.
4. **Validate**: Run `python .agent/skills/mobile-design-ios/scripts/ios_validator.py`.

---

## 🛠️ 4. Deterministic Verification Gate

Validate Swift code and verify Apple HIG compliance:

```powershell
# 1. Run built-in self-tests
python .agent/skills/mobile-design-ios/scripts/ios_validator.py --test

# 2. Audit a directory of Swift code
python .agent/skills/mobile-design-ios/scripts/ios_validator.py --audit-dir path/to/swift/sources
```