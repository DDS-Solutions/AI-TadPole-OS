-- Create used_nonces table for replay attack protection
CREATE TABLE IF NOT EXISTS used_nonces (
    nonce TEXT PRIMARY KEY NOT NULL,
    timestamp BIGINT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_nonces_timestamp ON used_nonces(timestamp);
