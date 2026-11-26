> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mobile-design-ios
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL_IOS_NAV]`)

# iOS Navigation Patterns (`NavigationStack` & `NavigationSplitView`)

---

## 1. Stack Navigation (`NavigationStack`)

In iOS 16+, `NavigationStack` manages push-and-pop stack navigation cleanly with value-based routing.

```swift
enum AppRoute: Hashable {
    case agentDetail(agentId: String)
    case securityAudit(logId: String)
}

struct MainTabView: View {
    @State private var navPath = NavigationPath()

    var body: some View {
        NavigationStack(path: $navPath) {
            List {
                NavigationLink(value: AppRoute.agentDetail(agentId: "agent-01")) {
                    Text("Agent 01 Telemetry")
                }
            }
            .navigationTitle("Overview")
            .navigationDestination(for: AppRoute.self) { route in
                switch route {
                case .agentDetail(let id):
                    AgentDetailView(agentId: id)
                case .securityAudit(let id):
                    SecurityAuditView(logId: id)
                }
            }
        }
    }
}
```

---

## 2. Multi-Column Adaptive Navigation (`NavigationSplitView`)

For iPadOS and larger iOS screens, use `NavigationSplitView`:

```swift
struct AdaptiveMainView: View {
    @State private var selectedAgentId: String?

    var body: some View {
        NavigationSplitView {
            List(agents, selection: $selectedAgentId) { agent in
                Text(agent.name)
            }
            .navigationTitle("Agents")
        } detail: {
            if let agentId = selectedAgentId {
                AgentDetailView(agentId: agentId)
            } else {
                ContentUnavailableView("Select an Agent", systemImage: "person.badge.shield")
            }
        }
    }
}
```

---

## 3. Modal Presentation Guidelines

| Presentation Type | Use Case | View Modifier |
|---|---|---|
| **Sheet** | Self-contained task or form | `.sheet(isPresented:)` |
| **Full Screen Cover** | Imperative workflow (e.g., camera scanner) | `.fullScreenCover(isPresented:)` |
| **Confirmation Dialog** | Destructive action confirmation | `.confirmationDialog(...)` |
| **Popover** | Contextual options (iPad / large screen) | `.popover(...)` |

```swift
.sheet(isPresented: $showScanner) {
    QrCodeScannerView(onScanned: { code in
        viewModel.handleQrCode(code)
        showScanner = false
    })
    .presentationDetents([.medium, .large])
    .presentationDragIndicator(.visible)
}
```