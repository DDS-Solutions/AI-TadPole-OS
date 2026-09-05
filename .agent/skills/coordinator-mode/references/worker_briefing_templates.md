> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / coordinator-mode
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Vague subagent prompt delegation or unhandled worker failures.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[COORDINATOR_MODE]`)

# Multi-Agent Worker Briefing Templates & Lifecycle (L3)

---

## 1. Concrete Worker Prompt Templates

### 🔍 Research Worker Briefing
```text
Task: Investigate [specific function/module] in [file/path].
Context: We are fixing [issue] because [rationale].
Scope: Inspect caller hierarchy and state interactions.
Output: Report concise bullet list of findings under 150 words.
```

### ⚡ Implementation Worker Briefing
```text
Task: Modify [target_file] lines [start_line]-[end_line].
Context: Replace [current_logic] with [new_logic] to satisfy [requirement].
Constraints: Do not modify shared interfaces without coordinator approval.
Verification: Confirm `npm run test` or `cargo check` passes.
```

---

## 2. Worker Fault Handling & Synthesis Rules

1. **Deadlock Avoidance**: Never spawn multiple writing workers on the same file path simultaneously.
2. **Subagent Failure Recovery**: If a worker subagent fails or times out, inspect logs, adjust constraints, and re-dispatch with explicit line-level guidance.