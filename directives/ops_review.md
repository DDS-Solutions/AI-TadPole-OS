> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[ops_review]` in audit logs.
>
> ### AI Assist Note
> 🛰️ Directive: Operations Review (SOP-OPS-01)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🛰️ Directive: Operations Review (SOP-OPS-01)

## 🎯 Primary Objective
Maintain peak operational efficiency for the Tadpole OS cluster. This directive governs the daily "pulse check" of the infrastructure, ensuring zero technical debt accumulation and absolute registry parity.

---

## 🔎 Verification Gates

### 1. Registry & Documentation Parity
- **Execution Script**: `python execution/parity_guard.py .`
- **Success Criteria**: 0 drift detected between `docs/*.md` and source code logic.
- **Remediation**: If drift is detected, immediately update the corresponding documentation using the `AI Assist Note` in the source file as the source of truth.

### 2. Performance Benchmarking
- **Execution Script**: `python execution/verify_all.py . --url http://localhost:8000`
- **KPIs**:
  - **Latency**: Mean response time < 800ms for utility skills.
  - **Uptime**: Gateway server must return 200 OK for `/health`.
  - **Context**: Verify `tiktoken` context pruning is active (no overflow errors).

### 3. Resource Health (Engine Pulse)
- **Telemetry Audit**: Check the Swarm Pulse (MessagePack stream) for high memory pressure (>85% RAM).
- **Action**: If pressure is high, trigger `memory_cleanup` background job and verify orphan workspace folders in `data/workspaces/` are purged.

### 4. Directives Compliance
- **Scan**: Audit all `directives/*.md` for "edit me" placeholders. 
- **Requirement**: The swarm can only function at "Sovereign" level if all active directives are fully populated with SOPs.

---

## 📊 Status Board Protocol
Update the `DASHBOARD_STATUS.md` artifact (if permitted) or generate a "Cluster Health Summary" as the final turn output. Include any active "Red Alerts" regarding budget exhaustion or provider connectivity faults.
[//]: # (Metadata: [ops_review])
