"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Test Verification Suite / test_cas
- **Primary Entrypoints**: `TestCas`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic internal state integrity and strict interface contract compliance.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[test_cas]`
- **Witness Tests**: none declared
"""

import os
import sqlite3
import unittest

class TestCas(unittest.TestCase):

    def setUp(self):
        print("\n[test_cas] Running CAS integrity test suite...")

    def test_cas_database_schema_and_deduplication(self):
        """Verify SQLite file_revisions migration, cas_blobs deduplication, backfill, and two-tier BLOB retrieval."""
        root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
        migration_file_v1 = os.path.join(root_dir, "migrations", "20260812000000_cas_file_revisions.sql")
        migration_file_v2 = os.path.join(root_dir, "migrations", "20260815000000_cas_blob_dedup.sql")

        self.assertTrue(os.path.exists(migration_file_v1), "Initial migration SQL file missing")
        self.assertTrue(os.path.exists(migration_file_v2), "CAS deduplication SQL file missing")

        with open(migration_file_v1, "r", encoding="utf-8") as f:
            sql_schema_v1 = f.read()

        with open(migration_file_v2, "r", encoding="utf-8") as f:
            sql_schema_v2 = f.read()

        # Test in in-memory SQLite database
        conn = sqlite3.connect(":memory:")
        cursor = conn.cursor()
        cursor.executescript(sql_schema_v1)

        # Insert historical revisions before migration to test backfill
        raw_bytes_1 = b"fn main() { println!(\"Hello CAS 1\"); }"
        hash_1 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

        raw_bytes_2 = b"fn main() { println!(\"Hello CAS 2\"); }"
        hash_2 = "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"

        cursor.execute(
            "INSERT INTO file_revisions (workspace_id, file_path, hash, content, size_bytes, version_num, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            ("/workspace", "src/main.rs", hash_1, sqlite3.Binary(raw_bytes_1), len(raw_bytes_1), 0, "2026-08-11T12:00:00Z")
        )
        cursor.execute(
            "INSERT INTO file_revisions (workspace_id, file_path, hash, content, size_bytes, version_num, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            ("/workspace", "src/main.rs", hash_2, sqlite3.Binary(raw_bytes_2), len(raw_bytes_2), 1, "2026-08-11T13:00:00Z")
        )
        # Duplicate hash in another revision (reverted back to v0)
        cursor.execute(
            "INSERT INTO file_revisions (workspace_id, file_path, hash, content, size_bytes, version_num, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            ("/workspace", "src/main.rs", hash_1, sqlite3.Binary(raw_bytes_1), len(raw_bytes_1), 2, "2026-08-11T14:00:00Z")
        )
        conn.commit()

        # Run Phase 2 deduplication migration (creates cas_blobs and backfills)
        cursor.executescript(sql_schema_v2)
        conn.commit()

        # Verify cas_blobs contains exactly 2 unique blobs for 3 revisions (deduplicated)
        cursor.execute("SELECT hash, size_bytes, content FROM cas_blobs ORDER BY hash")
        blobs = cursor.fetchall()
        self.assertEqual(len(blobs), 2, "cas_blobs should have exactly 2 distinct records for 2 unique hashes")

        # Verify two-tier join retrieval query
        cursor.execute("""
            SELECT r.version_num, r.hash, 
                   CASE WHEN b.content IS NOT NULL AND length(b.content) > 0 THEN b.content ELSE r.content END as content
            FROM file_revisions r
            LEFT JOIN cas_blobs b ON r.hash = b.hash
            WHERE r.workspace_id = ? AND r.file_path = ?
            ORDER BY r.version_num ASC
        """, ("/workspace", "src/main.rs"))

        joined_rows = cursor.fetchall()
        self.assertEqual(len(joined_rows), 3)
        self.assertEqual(bytes(joined_rows[0][2]), raw_bytes_1)
        self.assertEqual(bytes(joined_rows[1][2]), raw_bytes_2)
        self.assertEqual(bytes(joined_rows[2][2]), raw_bytes_1)

        conn.close()

if __name__ == "__main__":
    unittest.main()
