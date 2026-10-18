# Agent-to-Agent (A2A) Economics, Web3 Vault & Capability Architecture (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / a2a-protocol / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unauthorized budget overruns, invalid capability signatures, vault drainage.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document details the Agent-to-Agent (A2A) economic protocol, Web3 vault smart contracts, and cryptographic capability keys in Tadpole OS.

---

## 1. Capability Key Rotation & Authentication

Tadpole OS enforces zero-downtime key rotation using two overlapping keys:
- **`CAPABILITY_KEY_CURR`**: Active key used for issuing and signing current agent task capabilities.
- **`CAPABILITY_KEY_PREV`**: Grace period key accepted for verifying capabilities issued during the rotation window.

---

## 2. A2A Web3 Vault Security

- **`A2A_WEB3_VAULT_ADDRESS`**: Smart contract managing multi-agent escrow and transaction settlement.
- **`OVERSIGHT_PUBLIC_KEY`**: Cryptographic authority required to approve budget expansions exceeding `DEFAULT_AGENT_BUDGET_USD` or `DEFAULT_MISSION_BUDGET_USD`.
- **Two-Phase Commit (2PC)**: All economic settlements require hold-and-commit validation to prevent double-spending between concurrent subagents.
