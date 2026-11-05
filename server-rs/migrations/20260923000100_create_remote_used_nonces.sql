-- Keep paired-device request nonces separate from other timestamp formats.
CREATE TABLE IF NOT EXISTS remote_used_nonces (
    device_id TEXT NOT NULL,
    nonce TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (device_id, nonce)
);

CREATE INDEX IF NOT EXISTS idx_remote_used_nonces_timestamp
    ON remote_used_nonces(timestamp);

-- Preserve recently consumed paired-device nonces from the legacy shared table
-- during upgrade. Its other consumers use millisecond timestamps, so only copy
-- rows whose timestamp is in the remote protocol's seconds-based freshness window.
INSERT OR IGNORE INTO remote_used_nonces (device_id, nonce, timestamp)
SELECT paired_devices.id,
       SUBSTR(used_nonces.nonce, LENGTH(paired_devices.id) + 2),
       used_nonces.timestamp
FROM used_nonces
JOIN paired_devices
  ON SUBSTR(used_nonces.nonce, 1, LENGTH(paired_devices.id) + 1) = paired_devices.id || ':'
WHERE used_nonces.timestamp BETWEEN CAST(strftime('%s', 'now') AS INTEGER) - 300
                                AND CAST(strftime('%s', 'now') AS INTEGER) + 300
  AND LENGTH(SUBSTR(used_nonces.nonce, LENGTH(paired_devices.id) + 2)) BETWEEN 16 AND 128;
