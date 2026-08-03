> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[ios_hig_guidelines]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills:iOS**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL_IOS_HIG]` in audit logs.

# Apple Human Interface Guidelines (HIG) Reference

---

## 1. Layout & Touch Ergonomics

* **Minimum Hit Area:** All tap targets must measure at least **44 × 44 points**.
* **Thumb Zone:** Primary action buttons should reside in the lower two-thirds of the screen.
* **Safe Area Insets:** Always respect safe area bounds (`.ignoresSafeArea()` should only be applied to decorative backgrounds, maps, or full-bleed images).

---

## 2. Color, Materials & Dark Mode

* **System Materials:** Use SwiftUI background materials for translucent layers:
  - `.ultraThinMaterial`
  - `.thinMaterial`
  - `.regularMaterial`
* **Semantic System Colors:** Always use dynamic colors:
  - Backgrounds: `Color(uiColor: .systemBackground)`, `Color(uiColor: .secondarySystemBackground)`
  - Labels: `Color.primary`, `Color.secondary`

---

## 3. Typography & Dynamic Type

Use built-in text styles so fonts scale automatically when users adjust text size in iOS Settings:

```swift
Text("Agent Health Telemetry")
    .font(.title2)
    .fontWeight(.bold)

Text("Active background monitoring enabled")
    .font(.subheadline)
    .foregroundStyle(.secondary)
```

---

## 4. SF Symbols & Feedback

* Use SF Symbols 5+ with dynamic animations:
  ```swift
  Image(systemName: "shield.checkered")
      .symbolEffect(.bounce, value: isApproved)
  ```
* Provide sensory feedback on user action:
  ```swift
  Button("Sign & Approve") {
      approveAction()
  }
  .sensoryFeedback(.success, trigger: isApproved)
  ```

[//]: # (Metadata: [ios_hig_guidelines])
