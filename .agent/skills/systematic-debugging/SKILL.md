---
name: systematic-debugging
description: 4-phase systematic debugging methodology. Reproduce, Isolate, Understand, Fix.
allowed-tools: Read, Glob, Grep
---

# Systematic Debugging

**Stop guessing.**

## 4-Phase Process
1.  **Reproduce**: "Always? Often? Rare?". Steps to reproduce.
2.  **Isolate**: "When did it start? What changed? Minimal repro?".
3.  **Understand**: Root Cause Analysis (5 Whys).
4.  **Fix & Verify**: Verify fix + Regression test.

## Debugging Checklist
- [ ] Reproducible?
- [ ] Logs checked?
- [ ] Recent changes (`git log`)?
- [ ] Root cause found?
- [ ] Fix verified?
- [ ] Test added?

---

## 🧠 Aletheia Reasoning Protocol (Troubleshooting)

### 1. Reproduce
- **Tool**: `npm run test` or a dedicated reproduction mission.
- **Log Source**: Check `server-rs` stdout/stderr or `logs/error.log`.
- **Sidecar**: Check for "Tokio Runtime" initialization faults in the engine logs.

### 2. Isolate
- **Boundary**: Is it a Frontend Store fault (Zustand), a Backend Hub fault (Axum), or a Model provider fault (Ollama/LLM)?
- **Network**: Audit the browser's Network tab for WebSocket frame corruption or 429/500 API responses.

### 3. Understand
- **Audit Trace**: Review `tadpole.db` oversight logs for the specific `mission_id`. 
- **Code Audit**: Use `grep_search` to find all call-sites for the failing module.

### 4. Fix
- **Gate**: Ensure the fix adheres to the `clean-code` and `architecture` skills.
- **Verification**: Run `python execution/verify_all.py .` to ensure no regressions.

---

## 🛡️ Security & Safety Protocol (Debugging)

1.  **Sanitization**: Redact PII/Secrets in logs/shares.
2.  **Prod Access**: Read-only DB access.
3.  **Isolation**: No real customer data locally.
4.  **Cleanup**: Remove debug prints.
