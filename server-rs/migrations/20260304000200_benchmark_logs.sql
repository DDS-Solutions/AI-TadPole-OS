-- Migration to add benchmark_logs table for persistence stress testing
CREATE TABLE IF NOT EXISTS benchmark_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    test_id TEXT NOT NULL,
    step INTEGER NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);
