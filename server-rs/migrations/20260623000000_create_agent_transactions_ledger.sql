-- Migration: 20260623000000_create_agent_transactions_ledger.sql

-- 1. Agent Balance table: maps agent_id and asset/currency types to atomic u64 values
CREATE TABLE IF NOT EXISTS agent_balances (
    agent_id TEXT NOT NULL,
    asset_type TEXT NOT NULL, -- e.g. 'USDC', 'COMPUTE_TOKENS'
    balance INTEGER NOT NULL DEFAULT 0, -- u64 representation (stored as signed 64-bit integer in SQLite)
    PRIMARY KEY (agent_id, asset_type)
);

-- 2. Agent Asset Registry: tracks ownership of specific non-fungible or semi-fungible resources
CREATE TABLE IF NOT EXISTS agent_asset_registry (
    asset_id TEXT PRIMARY KEY,
    owner_agent_id TEXT NOT NULL,
    asset_name TEXT NOT NULL,
    asset_data TEXT -- JSON representation of metadata / capabilities
);

-- 3. Transaction Ledger: double-entry ledger signed with the audit key
CREATE TABLE IF NOT EXISTS transaction_ledger (
    id TEXT PRIMARY KEY,
    transaction_type TEXT NOT NULL, -- 'TRANSFER', 'BUY', 'SELL', 'FEE'
    buyer_address TEXT,
    seller_address TEXT,
    asset_id TEXT,
    amount INTEGER NOT NULL, -- u64 amount representation
    status TEXT NOT NULL, -- 'PENDING', 'COMMITTED', 'ROLLED_BACK'
    audit_signature TEXT, -- cryptographically signed transaction hash
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 4. Transaction Locks: tracks locked balances and assets for 2PC / Saga pattern
CREATE TABLE IF NOT EXISTS transaction_locks (
    lock_id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    buyer_id TEXT NOT NULL,
    seller_id TEXT NOT NULL,
    asset_id TEXT,
    locked_amount INTEGER NOT NULL,
    expires_at INTEGER NOT NULL, -- timestamp (seconds) after which lock can be rolled back
    FOREIGN KEY(transaction_id) REFERENCES transaction_ledger(id) ON DELETE CASCADE
);

-- 5. Economic Metadata: extends agents with zone status and limits
CREATE TABLE IF NOT EXISTS agent_economics_meta (
    agent_id TEXT PRIMARY KEY,
    economic_zone TEXT NOT NULL DEFAULT 'DEV', -- 'DEV', 'STAGING', 'PROD'
    daily_spend_limit INTEGER NOT NULL DEFAULT 0, -- u64 limit (0 = unlimited)
    daily_spent_accumulated INTEGER NOT NULL DEFAULT 0,
    last_reset_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
