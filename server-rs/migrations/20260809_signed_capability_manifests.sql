-- Migration: Cryptographically Signed Persistent Capability Manifests (SEC-05)
CREATE TABLE IF NOT EXISTS signed_capability_manifests (
    capability_id TEXT PRIMARY KEY,
    version TEXT NOT NULL,
    owner TEXT NOT NULL,
    creator TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    content_hash TEXT NOT NULL,
    signature TEXT NOT NULL,
    approval_id TEXT,
    expiration DATETIME,
    risk_class TEXT NOT NULL,
    resource_pattern TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'active', 'revoked', 'expired')) DEFAULT 'pending'
);

CREATE INDEX IF NOT EXISTS idx_signed_cap_owner ON signed_capability_manifests(owner, status);
CREATE INDEX IF NOT EXISTS idx_signed_cap_pattern ON signed_capability_manifests(risk_class, resource_pattern, status);
