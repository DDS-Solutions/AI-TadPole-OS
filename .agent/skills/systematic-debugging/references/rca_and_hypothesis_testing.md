# Root Cause Analysis (RCA) & Hypothesis Testing Protocols

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / systematic-debugging / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. The 4-Phase Debugging Methodology

```
[1. REPRODUCE] ───> [2. ISOLATE] ───> [3. ROOT CAUSE] ───> [4. VERIFY & ANNEAL]
 Create minimal,     Formulate binary  Trace point of       Prove green pass &
 fast (<2s) failing  hypotheses & run  divergence via       add regression test
 test or repro       active probes     telemetry map        to test suite
```

---

## 2. Hypothesis Discrimination Protocol

When isolating an unknown defect, formulate **two mutually exclusive hypotheses** rather than randomly altering code:

```markdown
### Hypothesis Discrimination Table
| Dimension | Hypothesis A ($H_A$) | Hypothesis B ($H_B$) |
|---|---|---|
| **Assumption** | Defect is in React client state hydration | Defect is in Tauri IPC payload deserialization |
| **Active Probe** | Inspect WebSocket wire payload in DevTools | Invoke IPC command directly via `cargo test` |
| **Expected if True** | Wire payload is valid; client parse fails | Rust handler returns error or truncated JSON |
| **Falsification Result** | Falsified if Rust handler fails independently | Falsified if Rust payload passes all unit checks |
```

---

## 3. The 5-Whys Root Cause Chain
1. **Symptom**: Task queue stalls intermittently on Windows.
2. **Why?**: Tokio worker thread pool becomes exhausted.
3. **Why?**: A file read in `routes/system.rs` blocked the worker thread.
4. **Why?**: `std::fs::read_to_string` was called inside an async function instead of `tokio::fs`.
5. **Root Cause**: Missing async lint rule allowing blocking `std::fs` inside async routes.
- **Permanent Fix**: Replace with `tokio::fs::read_to_string` and add `rust_auditor.py` to pre-commit lint gate.
