> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / testing-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Flaky mocks or non-deterministic test assertions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[TESTING_PATTERNS]`)

# Mocking & Fixture Patterns Reference (L3)

---

## 1. Test Double Taxonomy

| Test Double Type | Definition | Example Application |
|---|---|---|
| **Dummy** | Objects passed around but never actually used | Empty configuration struct passed to satisfier |
| **Stub** | Provides canned answers to calls during test | Mock HTTP client returning status 200 JSON |
| **Spy** | Records calls, arguments, and invocation counts | Verifying `send_task` was called with exact parameters |
| **Mock** | Object pre-programmed with expectations | Expecting `save_snapshot()` called exactly once |
| **Fake** | Working implementation with shortcuts | In-memory SQLite (`sqlite::memory:`) |

---

## 2. Multi-Language Test Fixture Templates

### TypeScript (Vitest)
```typescript
import { describe, it, expect, vi } from 'vitest';

describe('AgentService', () => {
  it('dispatches task with sanitized prompt', async () => {
    const mockApi = { sendTask: vi.fn().mockResolvedValue({ status: 'ok' }) };
    const res = await mockApi.sendTask('Clean prompt');
    expect(mockApi.sendTask).toHaveBeenCalledWith('Clean prompt');
    expect(res.status).toBe('ok');
  });
});
```

### Rust (`tokio::test`)
```rust
#[tokio::test]
async fn test_state_hub_dispatch() {
    let state = AppState::mock().await;
    let result = state.security.budget.check_quota("agent-1", 0.05).await;
    assert!(result.is_ok());
}
```