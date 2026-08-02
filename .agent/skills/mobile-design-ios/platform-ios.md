> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills:iOS**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL_IOS]` in audit logs.

# iOS SwiftUI Architecture & Platform Guide

---

## 1. SwiftUI Architecture (iOS 17+)

### Modern `@Observable` Data Flow
In iOS 17+, Apple introduced the `Observation` framework, replacing `ObservableObject` and `@Published`.

```swift
import SwiftUI
import Observation

// 1. Domain Model / State Holder
@Observable
final class ApprovalViewModel {
    var pendingApprovals: [ApprovalItem] = []
    var isLoading: Bool = false
    var errorMessage: String? = nil

    @ObservationIgnored
    private let repository: ApprovalRepository

    init(repository: ApprovalRepository = ApprovalRepository()) {
        self.repository = repository
    }

    @MainActor
    func fetchApprovals() async {
        isLoading = true
        defer { isLoading = false }
        do {
            pendingApprovals = try await repository.getPendingItems()
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
```

### View Consumption
```swift
struct ApprovalLedgerView: View {
    @State private var viewModel = ApprovalViewModel()

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading {
                    ProgressView("Loading approvals...")
                } else if viewModel.pendingApprovals.isEmpty {
                    ContentUnavailableView(
                        "No Pending Approvals",
                        systemImage: "checkmark.shield",
                        description: Text("All security requests have been processed.")
                    )
                } else {
                    List(viewModel.pendingApprovals) { item in
                        ApprovalRowView(item: item)
                    }
                }
            }
            .navigationTitle("Approval Ledger")
            .task {
                await viewModel.fetchApprovals()
            }
        }
    }
}
```

---

## 2. Concurrency & Swift Async/Await

* Always mark UI updates with `@MainActor`.
* Use `.task` view modifier to manage async work tied to view lifecycles (automatically cancels on view disappear).
* Use `Task.detached` only for heavy background computation off the main thread.

---

## 3. Storage & Persistence Guidelines

| Storage Layer | Use Case | Implementation |
|---|---|---|
| **SwiftData** | Structured local data, offline cache | `@Model`, `@Query` |
| **Keychain** | Auth tokens, private keys, API secrets | `SecItemAdd` / `Security` framework |
| **UserDefaults** | Non-sensitive user settings & preferences | `@AppStorage` |
