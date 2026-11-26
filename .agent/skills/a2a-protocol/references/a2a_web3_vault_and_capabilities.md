# Agent-to-Agent (A2A) Economics, Web3 Vault & Capability Architecture (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / a2a-protocol / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unauthorized budget overruns, invalid capability signatures, vault drainage.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document details the Agent-to-Agent (A2A) economic protocol, Web3 vault smart contracts, cryptographic capability keys, and the transition path from local simulation to live on-chain mainnet settlement in AI-Tadpole-OS.

---

## 1. Capability Key Rotation & Authentication

AI-Tadpole-OS enforces zero-downtime key rotation using two overlapping keys:
- **`CAPABILITY_KEY_CURR`**: Active key used for issuing and signing current agent task capabilities.
- **`CAPABILITY_KEY_PREV`**: Grace period key accepted for verifying capabilities issued during the rotation window.

---

## 2. A2A Web3 Vault Security (Operational Modes)

### Mode A: Off-Chain Simulation (Default / Current)
- **Local Ledger**: High-speed, microsecond-latency 2PC transactions in SQLite `transaction_ledger` and `transaction_locks`.
- **Hybrid Adapter**: `L3HybridAdapter` returns deterministic `0xsim_<uuid>` receipts when `A2A_WEB3_RPC_URL` is empty, `sim://`, or `mock://`.
- **Fail-Closed Gate**: If pointed to a live external RPC (`http://` or `https://`) without an authenticated signer bridge, the adapter rolls back the lock and rejects the transfer with an explicit `BadRequest` error.

### Mode B: Live On-Chain Settlement (Planned / Roadmap `A2E-02`)
The transition from simulated exits to live EVM settlement requires four production pillars:

#### Pillar 1: EVM Transaction Signer Bridge
- **Rust Dependencies**: Add `alloy` (preferred for modern async Tokio/Axum) or `ethers-rs` to `server-rs/Cargo.toml`.
- **Signer Providers**: Support either:
  1. Local HSM / Encrypted Keypair: Loaded from secure memory (`secrecy::SecretString`) via `A2A_WEB3_PRIVATE_KEY`.
  2. Cloud KMS / MPC Signer: AWS KMS or Google Cloud KMS bridge for enterprise Zero Trust key management.
- **Gas Strategy**: Dynamic EIP-1559 gas price calculation (`max_fee_per_gas`, `max_priority_fee_per_gas`) with hard-cap ceilings to prevent fee drainage.

#### Pillar 2: Escrow Smart Contract (`A2AEscrowVault.sol`)
Deploy a dedicated multi-agent escrow vault smart contract to target EVM networks (e.g., Base, Arbitrum, Ethereum):
```solidity
// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.24;

interface IA2AEscrowVault {
    event FundsDeposited(address indexed from, string indexed agentId, uint256 amount);
    event SettlementExecuted(bytes32 indexed lockId, address indexed recipient, uint256 amount);
    event SettlementRefunded(bytes32 indexed lockId, address indexed depositor, uint256 amount);

    function deposit(string calldata agentId, uint256 amount) external;
    function executeSettlement(
        bytes32 lockId,
        address recipient,
        uint256 amount,
        bytes calldata oversightSignature
    ) external;
    function refundSettlement(bytes32 lockId) external;
}
```

#### Pillar 3: Inbound Blockchain Event Listener (Deposit Oracle)
- **Background Worker**: A Tokio service running `eth_subscribe` (WebSocket) or polling `eth_getLogs` on a configurable block interval.
- **Event Decoding**: Watches for `FundsDeposited` events from `A2A_WEB3_VAULT_ADDRESS`.
- **Atomic Balance Ingestion**: Automatically credits the local SQLite `agent_balances` record matching the decoded `agentId` with ALCOA+ cryptographic audit trail logging.

#### Pillar 4: Security & Circuit Breakers
- **Oversight Key Gate**: Enforce `OVERSIGHT_PUBLIC_KEY` Ed25519 or secp256k1 signature validation for all on-chain settlements exceeding `DEFAULT_AGENT_BUDGET_USD` ($50.00).
- **Idempotency & Replay Defense**: On-chain nonces coupled with SQLite `consumed_invoices` tracking to guarantee each 2PC `lock_id` can be settled exactly once.
- **Gas Burn Velocity Limit**: Circuit breaker that halts automated on-chain exits if total daily gas expenditure exceeds 5% of transferred value.

---

## 3. Environment Configuration for Live Mode (Option B)

| Environment Variable | Type | Description |
| :--- | :--- | :--- |
| `A2A_WEB3_RPC_URL` | URL | HTTP/WebSocket RPC provider (e.g., Alchemy/Infura on Base/Arbitrum). |
| `A2A_WEB3_VAULT_ADDRESS` | Hex | Contract address of deployed `A2AEscrowVault`. |
| `A2A_WEB3_SIGNER_MODE` | String | `local_key`, `aws_kms`, or `gcp_kms`. |
| `A2A_WEB3_PRIVATE_KEY` | Hex | Required if `local_key` mode (32-byte EVM private key). |
| `A2A_WEB3_CHAIN_ID` | Integer | Target EVM chain ID (e.g., `8453` for Base Mainnet). |
| `A2A_WEB3_MAX_GAS_PRICE_GWEI` | Integer | Hard safety ceiling for EIP-1559 gas fee spikes. |
