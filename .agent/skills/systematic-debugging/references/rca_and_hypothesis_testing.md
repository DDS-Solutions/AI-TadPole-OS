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

## 2. The 3-Fix Circuit Breaker Protocol

When working through bugs or test failures:
- **Iteration 1**: Formulate $H_1$, execute targeted fix, run repro. If fails $\rightarrow$ Revert.
- **Iteration 2**: Formulate $H_2$ based on new evidence, execute fix, run repro. If fails $\rightarrow$ Revert.
- **Iteration 3**: Formulate $H_3$ based on deeper trace, execute fix, run repro. If fails $\rightarrow$ Revert.
- **Circuit Breaker Trip**: **DO NOT ATTEMPT ITERATION 4**.
  - Halt and document:
    1. Symptoms and reproduction command.
    2. The 3 hypotheses tested and why each was falsified.
    3. Underlying architectural inconsistency or ambiguity requiring Entity 0 guidance.

---

## 3. Hypothesis Discrimination & Bisection Ledger

Before modifying code, log hypotheses into a formal ledger:

```markdown
### Hypothesis Discrimination Ledger
| ID | Layer Boundary | Hypothesis | Prediction | Verification Command | Outcome |
|---|---|---|---|---|---|
| H1 | React UI | State hydration race | Store value missing before mount | `console.log(useStore.getState())` | Falsified: Store hydrated |
| H2 | Tauri IPC | Serialized string truncation | Rust handler receives malformed JSON | `cargo test test_ipc_command` | Falsified: Rust receives valid JSON |
| H3 | Axum Engine | Mutex held across await | Tokio thread deadlock on channel | Check trace spans & thread pool | Confirmed: Mutex contention |
```

---

## 4. Multi-Layer System Bisection Sequence

In Tadpole OS, isolate faults by dividing across physical boundaries:
1. **Layer 1: Frontend UI**: Zustand store, React components, DOM events.
2. **Layer 2: IPC Boundary**: Tauri `invoke()` calls, `@msgpack/msgpack` serialization, capabilities.
3. **Layer 3: Core Engine**: Rust Axum routes, Tokio tasks, state hubs (`comm`, `gov`, `reg`, `res`, `sec`).
4. **Layer 4: Storage**: SQLite transactions (`sqlx`), LanceDB vector tables, disk files.

Always verify boundaries in sequence to isolate which layer introduces the divergence.

---

## 5. The 5-Whys Root Cause Chain

1. **Symptom**: Task queue stalls intermittently on Windows.
2. **Why?**: Tokio worker thread pool becomes exhausted.
3. **Why?**: A file read in `routes/system.rs` blocked the worker thread.
4. **Why?**: `std::fs::read_to_string` was called inside an async function instead of `tokio::fs`.
5. **Root Cause**: Missing async lint rule allowing blocking `std::fs` inside async routes.
- **Permanent Fix**: Replace with `tokio::fs::read_to_string` and add `rust_auditor.py` to pre-commit lint gate.
