> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mobile-design-ios
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL_IOS_PERF]`)

# iOS Performance & Testing Guidelines

---

## 1. 120Hz ProMotion & Render Optimization

### Minimizing Body Re-evaluations
* Keep SwiftUI `@State` scoped locally to small child views.
* Use `@Observable` (iOS 17+) over `ObservableObject` so SwiftUI only re-evaluates views that read modified properties.
* Avoid performing heavy computations inside the `body` property of views; pre-compute values in ViewModel or using `.task`.

### Lazy Loading Lists
* Use `LazyVStack` or `List` for dynamic data streams. Avoid placing large numbers of child views inside plain `VStack` or `ScrollView`.

```swift
ScrollView {
    LazyVStack(spacing: 12) {
        ForEach(logs) { log in
            LogEntryRow(log: log)
        }
    }
}
```

---

## 2. Unit & Integration Testing (Swift Testing)

In Xcode 16+, Apple introduced the `@Test` macros for modern Swift testing:

```swift
import Testing
@testable import TadpoleCompanion

struct ApprovalViewModelTests {
    @Test("ViewModel successfully processes pending approval items")
    func testFetchApprovalsSuccess() async throws {
        let mockRepo = MockApprovalRepository(items: [
            ApprovalItem(id: "req-101", action: "DEPLOY_SWARM", status: .pending)
        ])
        let viewModel = ApprovalViewModel(repository: mockRepo)

        await viewModel.fetchApprovals()

        #expect(viewModel.pendingApprovals.count == 1)
        #expect(viewModel.pendingApprovals.first?.id == "req-101")
        #expect(viewModel.errorMessage == nil)
    }
}
```

---

## 3. Profiling with Xcode Instruments

* **Time Profiler**: Identify main-thread bottlenecks causing frame drops below 60fps/120fps.
* **SwiftUI Instrument**: Track render pass counts, view body duration, and state churn.
* **Leaks Instrument**: Detect retain cycles (e.g. escaping closures holding strong `self` references).