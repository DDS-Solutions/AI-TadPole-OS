-- Migration: 20260824000000_a2a_hardening.sql
-- Hardening indexes and replay protection tables for A2A subsystem

CREATE TABLE IF NOT EXISTS consumed_invoices (
    invoice_id TEXT PRIMARY KEY,
    consumed_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_transaction_locks_buyer ON transaction_locks(buyer_id);
CREATE INDEX IF NOT EXISTS idx_transaction_locks_expires ON transaction_locks(expires_at);
CREATE INDEX IF NOT EXISTS idx_agent_directives_target_status ON agent_directives(target_agent_id, status);
