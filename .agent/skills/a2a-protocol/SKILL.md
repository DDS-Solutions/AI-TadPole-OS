---
name: a2a-protocol
description: "Agent-to-Agent (A2A) economic protocol, Web3 vault settlement, capability key rotation, and cryptographic oversight signatures."
when_to_use: "When dispatching paid subagent tasks, handling Web3 vault transactions, managing capability key rotation, or verifying oversight authorization."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / a2a-protocol
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unauthorized budget overruns, invalid capability signatures, vault drainage.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Agent-to-Agent (A2A) Economic & Capability Protocol

> **Purpose**: Govern inter-agent economic settlements, cryptographic capability delegation, and budget limits across the swarm.
> **Workflow Binding**: Used during [`/orchestrate`](../../workflows/orchestrate.md) and [`/audit`](../../workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/a2a_web3_vault_and_capabilities.md`](./references/a2a_web3_vault_and_capabilities.md) | Web3 vault smart contract specs, key rotation window, hold-and-commit 2PC | Dispatching paid subagent missions or rotating keys |
| [`scripts/capability_key_validator.py`](./scripts/capability_key_validator.py) | Deterministic budget cap validator and capability key format checker | Pre-dispatch mission budgeting or self-test verification |

---

## 🏛️ Core Architectural Invariants

1. **Two-Key Rotation Grace Period**: The system must accept both `CAPABILITY_KEY_CURR` and `CAPABILITY_KEY_PREV` during scheduled key rotation windows.
2. **Oversight Key Gate**: Any subagent task demanding budget $> \$50.00$ USD requires an explicit cryptographic signature from `OVERSIGHT_PUBLIC_KEY`.
3. **Escrow Hold-and-Commit**: Web3 vault settlements must freeze funds on task dispatch and release on proof-of-work completion.

---

## 🛡️ Socratic Gate: Budget Authority & Cryptographic Verification

Before committing inter-agent financial transactions:
1. **Budget Allowance**: Is the requested budget within the agent's authorized quota?
2. **Key Freshness**: Is the capability key active or within the grace period?
3. **Idempotency**: Is the transaction keyed with a unique UUID to prevent double-spending?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Ambient unmetered agent spending | Rapid wallet drainage by runaway loops | Enforce per-mission caps (`DEFAULT_MISSION_BUDGET_USD`) |
| Instant single-key rotation without grace | Subagents in flight fail mid-task | Support dual-key verification (`CURR` + `PREV`) |
| Unsigned budget expansions | Security bypass of financial governance | Reject budgets $> \$50.00$ without oversight signature |

---

## 🛠️ Verification & Execution Commands

```powershell
# Validate mission budget
python .agent/skills/a2a-protocol/scripts/capability_key_validator.py --budget 25.0

# Run self-test suite
python .agent/skills/a2a-protocol/scripts/capability_key_validator.py --test
```
