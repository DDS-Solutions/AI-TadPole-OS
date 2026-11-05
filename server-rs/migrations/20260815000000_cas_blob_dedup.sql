-- Two-tier Content-Addressable Storage (CAS) Deduplication
-- Separates content BLOB storage by SHA-256 hash to eliminate redundant binary copies across revisions

CREATE TABLE IF NOT EXISTS cas_blobs (
    hash TEXT PRIMARY KEY,               -- SHA-256 content digest
    content BLOB NOT NULL,               -- Deduplicated binary payload
    size_bytes INTEGER NOT NULL,         -- Payload size
    created_at TEXT NOT NULL             -- First indexed timestamp
);

-- Backfill from existing historical revisions
INSERT OR IGNORE INTO cas_blobs (hash, content, size_bytes, created_at)
SELECT hash, content, size_bytes, MIN(created_at)
FROM file_revisions
WHERE content IS NOT NULL
GROUP BY hash;
